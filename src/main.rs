use bktree::BkTree;
use eyre::{
    Context,
    Result,
};
use image::io::Reader as ImageReader;
use img_hash::{
    HashAlg,
    HasherConfig,
    ImageHash,
};
use rayon::prelude::*;
use std::{
    cmp::Ordering,
    collections::HashSet,
    ffi::OsString,
    fs::{
        self,
        File,
    },
    io::{
        Cursor,
        Read,
    },
    path::{
        Path,
        PathBuf,
    },
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(short, long, parse(from_os_str))]
    old: PathBuf,

    #[structopt(short, long, parse(from_os_str))]
    new: PathBuf,

    #[structopt(short, long)]
    distance: u8,
}

#[derive(Debug, Eq)]
struct Page {
    path: PathBuf,
    size: usize,
}

impl Ord for Page {
    fn cmp(&self, other: &Self) -> Ordering {
        self.path.cmp(&other.path)
    }
}

impl PartialOrd for Page {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Page {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

#[derive(Clone, Debug)]
struct HashedImage {
    filename: OsString,
    index: usize,
    hash: ImageHash,
}

struct Match {
    src: HashedImage,
    dst: Option<(HashedImage, isize)>,
}

fn image_distance(img1: &HashedImage, img2: &HashedImage) -> isize {
    img1.hash.dist(&img2.hash) as isize
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let old = hash_images(&opts.old)
        .wrap_err_with(|| format!("hashing {}", opts.old.display()))?;
    let new = hash_images(&opts.new)
        .wrap_err_with(|| format!("hashing {}", opts.new.display()))?;

    let mut hashes = BkTree::new(image_distance);
    hashes.insert_all(old);

    let mut missing = hashes
        .iter()
        .map(|image| &image.filename)
        .collect::<HashSet<_>>();

    let mapping = new
        .into_iter()
        .map(|image| {
            let matches = hashes.find(image.clone(), opts.distance.into());
            match matches
                .into_iter()
                .filter(|(image, _)| missing.contains(&image.filename))
                .min_by_key(|(img, dist)| {
                    *dist
                        + (img.index as isize - image.index as isize).abs() / 5
                }) {
                Some((matching, distance)) => {
                    missing.remove(&matching.filename);
                    Match {
                        src: image,
                        dst: Some((matching.clone(), distance)),
                    }
                },
                None => {
                    Match {
                        src: image,
                        dst: None,
                    }
                },
            }
        })
        .collect::<Vec<_>>();

    println!("PAGE MAPPING:");
    for m in mapping {
        match m.dst {
            Some((image, distance)) => {
                println!(
                    "\t{} MATCH {} (DISTANCE: {})",
                    opts.new.join(m.src.filename).display(),
                    opts.old.join(image.filename).display(),
                    distance
                )
            },
            None => {
                println!(
                    "\t{} (NEW PAGE)",
                    opts.new.join(m.src.filename).display()
                )
            },
        }
    }

    if !missing.is_empty() {
        println!("\nMISSING PAGES");
        for filename in missing {
            println!("\t{}", opts.old.join(filename).display())
        }
    }
    Ok(())
}

fn list_pages(path: &Path) -> Result<Vec<Page>> {
    fs::read_dir(path)
        .wrap_err("list pages")?
        .filter_map(|entry| {
            entry
                .wrap_err("access directory entry")
                .and_then(|entry| {
                    entry
                        .metadata()
                        .wrap_err_with(|| {
                            format!(
                                "read metadata for {}",
                                entry.path().display()
                            )
                        })
                        .map(|metadata| {
                            metadata.is_file().then(|| {
                                Page {
                                    path: entry.path(),
                                    size: metadata.len() as usize,
                                }
                            })
                        })
                })
                .transpose()
        })
        .collect::<Result<Vec<_>>>()
}

fn hash_images(path: impl Into<PathBuf>) -> Result<Vec<HashedImage>> {
    let path = path.into();
    println!("Hashing pages from {}…", path.display());

    let mut pages = list_pages(&path)?;
    pages.sort();

    pages
        .into_par_iter()
        .enumerate()
        .try_fold(Vec::new, |mut acc, (index, page)| {
            let filename =
                page.path.file_name().expect("missing filename").to_owned();
            let mut file = File::open(&page.path)?;
            let mut contents = Vec::with_capacity(page.size);
            file.read_to_end(&mut contents).wrap_err_with(|| {
                format!("cannot read page {}", page.path.display())
            })?;

            let image = ImageReader::new(Cursor::new(contents))
                .with_guessed_format()
                .wrap_err_with(|| {
                    format!("identify {}", filename.to_string_lossy())
                })?
                .decode()
                .wrap_err_with(|| {
                    format!("decode {}", filename.to_string_lossy())
                })?;

            let hasher = HasherConfig::new()
                .hash_size(8, 8)
                .hash_alg(HashAlg::DoubleGradient)
                .preproc_dct()
                .to_hasher();

            acc.push(HashedImage {
                filename,
                index,
                hash: hasher.hash_image(&image),
            });

            Ok(acc)
        })
        .try_reduce(Vec::new, |mut v1, v2| {
            v1.extend(v2.into_iter());
            Ok(v1)
        })
}

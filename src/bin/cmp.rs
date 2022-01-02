//! A small tool to check which algorithm would work best on two given images.
//!
//! A small tool to check which algorithm would work best on two given images.
//!
//! Try different perceptual hashes, with and without pre-processing and print
//! the distance between the two images.
//! Useful to understand how sensible is a given algorithm and/or how important
//! is the pre-processing.
use eyre::{Context, Result};
use image::{io::Reader as ImageReader, DynamicImage};
use img_hash::{HashAlg, HasherConfig};
use std::{
    fs::File,
    io::{Cursor, Read},
    path::PathBuf,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    #[structopt(parse(from_os_str))]
    first: PathBuf,

    #[structopt(parse(from_os_str))]
    second: PathBuf,
}

fn main() -> Result<()> {
    let opts = Opts::from_args();
    let first = load_image(opts.first)?;
    let second = load_image(opts.second)?;

    hash_with_mean(&first, &second);
    hash_with_gradient(&first, &second);
    hash_with_vgradient(&first, &second);
    hash_with_dgradient(&first, &second);
    hash_with_blockhash(&first, &second);

    Ok(())
}

fn load_image(path: impl Into<PathBuf>) -> Result<DynamicImage> {
    let path = path.into();
    let filename = path.file_name().expect("missing filename").to_owned();
    let mut file = File::open(&path)?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents)
        .wrap_err_with(|| format!("cannot read image {}", path.display()))?;

    ImageReader::new(Cursor::new(contents))
        .with_guessed_format()
        .wrap_err_with(|| format!("identify {}", filename.to_string_lossy()))?
        .decode()
        .wrap_err_with(|| format!("decode {}", filename.to_string_lossy()))
}

fn hash_with_mean(first: &DynamicImage, second: &DynamicImage) {
    let hasher = HasherConfig::new().hash_alg(HashAlg::Mean).to_hasher();
    let dist = hasher.hash_image(first).dist(&hasher.hash_image(second));

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Mean)
        .preproc_dct()
        .to_hasher();
    let dist_dct = hasher.hash_image(first).dist(&hasher.hash_image(second));

    println!("Algo: Mean, dist: {} (w/o DCT: {})", dist_dct, dist);
}

fn hash_with_gradient(first: &DynamicImage, second: &DynamicImage) {
    let hasher = HasherConfig::new().hash_alg(HashAlg::Gradient).to_hasher();
    let dist = hasher.hash_image(first).dist(&hasher.hash_image(second));

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Gradient)
        .preproc_dct()
        .to_hasher();
    let dist_dct = hasher.hash_image(first).dist(&hasher.hash_image(second));

    println!("Algo: Gradient, dist: {} (w/o DCT: {})", dist_dct, dist);
}

fn hash_with_vgradient(first: &DynamicImage, second: &DynamicImage) {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::VertGradient)
        .to_hasher();
    let dist = hasher.hash_image(first).dist(&hasher.hash_image(second));

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::VertGradient)
        .preproc_dct()
        .to_hasher();
    let dist_dct = hasher.hash_image(first).dist(&hasher.hash_image(second));

    println!("Algo: VertGradient, dist: {} (w/o DCT: {})", dist_dct, dist);
}

fn hash_with_dgradient(first: &DynamicImage, second: &DynamicImage) {
    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .to_hasher();
    let dist = hasher.hash_image(first).dist(&hasher.hash_image(second));

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::DoubleGradient)
        .preproc_dct()
        .to_hasher();
    let dist_dct = hasher.hash_image(first).dist(&hasher.hash_image(second));

    println!(
        "Algo: DoubleGradient, dist: {} (w/o DCT: {})",
        dist_dct, dist
    );
}

fn hash_with_blockhash(first: &DynamicImage, second: &DynamicImage) {
    let hasher = HasherConfig::new().hash_alg(HashAlg::Blockhash).to_hasher();
    let dist = hasher.hash_image(first).dist(&hasher.hash_image(second));

    let hasher = HasherConfig::new()
        .hash_alg(HashAlg::Blockhash)
        .preproc_diff_gauss()
        .to_hasher();
    let dist_dct = hasher.hash_image(first).dist(&hasher.hash_image(second));

    println!("Algo: Blockhash, dist: {} (w/o DCT: {})", dist_dct, dist);
}

# img-match

[![License](https://img.shields.io/badge/License-BSD%203--Clause-blue.svg)](https://opensource.org/licenses/BSD-3-Clause)

img-match is a command-line tool that allows you to compare pages of a two
versions of a book. It allows you to find missing pages, check pages order, …

## How to install

You can download a pre-compiled executable for Linux, MacOS and Windows
operating systems
[on the release page](https://github.com/TehUncleDolan/img-match/releases/latest),
then you should copy that executable to a location from your `$PATH` env.

You might need to run `chmod +x img-match_amd64` or `chmod +x img-match_darwin`.

## Usage

The simplest invocation only requires you to specify the directories where the
files you want to compare are.

```bash
img-match --old my-book-v1 --new my-book-v2 --distance 12
```

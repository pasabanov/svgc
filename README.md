# svgc

[![crates.io](https://img.shields.io/crates/v/svgc?style=for-the-badge)](https://crates.io/crates/svgc)

Rust version of the [SvgCompress](https://github.com/pasabanov/SvgCompress/) tool.

## Description

`svgc` is a tool for compressing SVG files by removing unnecessary whitespace, comments, metadata and some other data. It also supports optimization with [SVGO](https://github.com/svg/svgo) and compression into [SVGZ](https://ru.wikipedia.org/wiki/SVG#SVGZ). The tool helps reduce the file size and clean up SVG files for better performance and preparing for release versions.

## Installation

#### Dependencies

[Cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) needs to be installed in order to install or build the utility.

(Optional) If you want to use `--svgo` option, install [SVGO](https://github.com/svg/svgo) as well.

### With Cargo (recommended):

```sh
cargo install svgc
```

### From GitHub (build yourself):

```sh
git clone https://github.com/pasabanov/svgc
cd svgc
cargo build --profile release
```

The built file will be located in the `target/release` directory.

## Usage

To compress SVG files, run the script with the following command:

```sh
svgc [options] paths
```

## Options

`-h`, `--help` Show this help message and exit  
`-v`, `--version` Show the version of the program  
`-r`, `--recursive` Recursively process directories  
`-f`, `--remove-fill` Remove `fill="..."` attributes  
`-o`, `--svgo` Use [SVGO](https://github.com/svg/svgo) if it is installed on the system  
`-z`, `--svgz` Compress to [.svgz](https://ru.wikipedia.org/wiki/SVG#SVGZ) format after optimization  
`-n`, `--no-default` Do not perform default optimizations (if you only want to use [SVGO](https://github.com/svg/svgo), compress to .svgz, or perform both operations)  
`-q`, `--quiet` Only output error messages, not regular status messages

## Examples

1. Compress a single SVG file:
	```sh
	svgc my-icon.svg
	```
2. Compress all SVG files in some directories and files:
	```sh
	svgc my-icons-directory1 my-icon.svg directory2 icon2.svg
	```
3. Compress all SVG files in a directory and all subdirectories:
	```sh
	svgc -r my-icons-directory
   ```
4. Compress an SVG file removing every `fill=...` attribute in it (making it monocolor):
	```sh
	svgc -f my-icon.svg
	```
5. Compress all SVG files in a directory and all subdirectories, removing `fill` attributes, then optimize with SVGO, then compress to .svgz:
	```sh
	svgc -rfoz my-icons-directory
	```

## License

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.

## Copyright

2024 Petr Alexandrovich Sabanov

## Metrics

![repo size](https://img.shields.io/github/repo-size/pasabanov/svgc?color=6e54bb)
![crate size](https://img.shields.io/crates/size/svgc?label=crate%20size&color=orange)
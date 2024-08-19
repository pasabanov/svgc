//! svgc is a tool for compressing SVG files
//! Copyright (C) © 2024  Petr Alexandrovich Sabanov
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU General Public License for more details.
//!
//! You should have received a copy of the GNU General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io;
use std::fs;
use std::path::{PathBuf};

use clap::{Arg, ArgAction::SetTrue, Command};

mod default_opt;
mod svgo;
mod svgz;
mod files;

use default_opt::default_optimize;
use svgo::run_svgo;
use svgz::compress_to_svgz;
use files::find_svg_files;

fn main() -> io::Result<()> {
	let matches = Command::new("svgc")
		.version("0.1.5")
		.long_version(
			"0.1.5\n\
			Copyright (C) 2024 Petr Alexandrovich Sabanov.\n\
			License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\n\
			This is free software: you are free to change and redistribute it.\n\
			There is NO WARRANTY, to the extent permitted by law."
		)
		.about("Compress SVG files by removing unnecessary whitespace, comments, metadata, and some other data.")
		.arg(Arg::new("paths").help("List of SVG files or directories or SVG files to compress")
			.required(true)
			.num_args(1..))
		.arg(Arg::new("recursive")  .short('r').long("recursive")  .help("Recursively process directories")
			.action(SetTrue))
		.arg(Arg::new("remove-fill").short('f').long("remove-fill").help("Remove fill=\"...\" attributes")
			.action(SetTrue))
		.arg(Arg::new("svgo")       .short('o').long("svgo")       .help("Use SVGO if it exists in the system")
			.action(SetTrue))
		.arg(Arg::new("svgz")       .short('z').long("svgz")       .help("Compress to .svgz format with gzip utility after processing")
			.action(SetTrue))
		.arg(Arg::new("no-default") .short('n').long("no-default") .help("Skip default optimizations")
			.action(SetTrue))
		.arg(Arg::new("no-backup")  .short('B').long("no-backup")  .help("Do not create backup (.bak) files")
			.action(SetTrue))
		.get_matches();

	let paths: Vec<PathBuf> = matches.get_many::<String>("paths").unwrap().map(PathBuf::from).collect();
	let recursive = matches.get_flag("recursive");
	let remove_fill = matches.get_flag("remove-fill");
	let use_svgo = matches.get_flag("svgo");
	let compress_svgz = matches.get_flag("svgz");
	let no_default = matches.get_flag("no-default");
	let no_backup = matches.get_flag("no-backup");

	if paths.is_empty() || no_default && !use_svgo && !compress_svgz {
		return Ok(())
	}

	let svgo_path = if use_svgo {
		match which::which("svgo") {
			Ok(path) => Some(path.display().to_string()),
			Err(_) => {
				eprintln!("Error: svgo executable not found in the system.");
				None
			}
		}
	} else {
		None
	};

	let mut svg_files = vec!();

	for path in paths {
		find_svg_files(&mut svg_files, &path, recursive)?
	}

	if !no_backup {
		for file in &svg_files {
			let backup_file = file.with_extension(".bak");
			if backup_file.exists() {
				return Err(io::Error::new(
					io::ErrorKind::AlreadyExists,
					"Backup file already exists"
				))
			}
			fs::copy(file, backup_file)?;
		}
	}

	if !no_default {
		for file in &svg_files {
			default_optimize(file, remove_fill)?;
		}
	}

	if use_svgo && svgo_path != None {
		run_svgo(&svg_files, &svgo_path.unwrap())?;
	}

	if compress_svgz {
		for file in &svg_files {
			compress_to_svgz(file)?;
		}
	}

	Ok(())
}
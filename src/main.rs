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

use std::collections::HashSet;
use std::fs;
use std::path::{PathBuf};
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgAction::SetTrue, Command};

mod default_opt;
mod files;
mod svgo;
mod svgz;

use default_opt::default_optimize_files;
use files::TempBackupStorage;
use svgo::run_svgo;
use svgz::compress_to_svgz;

fn main() -> ExitCode {
	let matches = Command::new("svgc")
		.about("Compress SVG files by removing unnecessary whitespace, comments, metadata, and some other redundant data.\n\
				Optionally, you can use SVGO for additional optimization, and compress the files to .svgz format.\n\
				\n\
				This program DOES NOT create backups (outside of the program's lifetime) if it runs successfully, so use it carefully!\n\
				If an error occurs, backups of the original files will be saved.")
		.version("0.1.6")
		.long_version(
			"0.1.6\n\
			Copyright (C) 2024 Petr Alexandrovich Sabanov.\n\
			License GPLv3+: GNU GPL version 3 or later <https://gnu.org/licenses/gpl.html>.\n\
			This is free software: you are free to change and redistribute it.\n\
			There is NO WARRANTY, to the extent permitted by law."
		)
		.arg(Arg::new("paths").help("List of SVG files or directories containing SVG files to compress")
			.required(true)
			.num_args(1..))
		.arg(Arg::new("recursive")  .short('r').long("recursive")  .help("Recursively process directories")
			.action(SetTrue))
		.arg(Arg::new("remove-fill").short('f').long("remove-fill").help("Remove fill=\"...\" attributes")
			.action(SetTrue))
		.arg(Arg::new("svgo")       .short('o').long("svgo")       .help("Use SVGO if it is installed on the system")
			.action(SetTrue))
		.arg(Arg::new("svgz")       .short('z').long("svgz")       .help("Compress to .svgz format after optimization")
			.action(SetTrue))
		.arg(Arg::new("no-default") .short('n').long("no-default") .help("Do not perform default optimizations")
			.action(SetTrue))
		.arg(Arg::new("quiet")      .short('q').long("quiet")      .help("Only output error messages, not regular status messages")
			.action(SetTrue))
		.disable_version_flag(true)
		.arg(Arg::new("version")    .short('v').long("version")    .help("Print version").action(ArgAction::Version))
		.disable_help_flag(true)
		.arg(Arg::new("help")       .short('h').long("help")       .help("Print help")   .action(ArgAction::Help))
		.get_matches();

	let paths: Vec<PathBuf> =
		matches
			.get_many::<String>("paths")
			.unwrap()
			.map(PathBuf::from)
			.filter_map(|path| {
				match fs::canonicalize(&path) {
					Ok(canon) => {
						if canon.is_dir() || canon.is_file() && canon.extension().unwrap_or_default().eq("svg") {
							Some(canon)
						} else {
							eprintln!("The path {} is not a valid SVG file or directory.", path.display());
							None
						}
					}
					Err(e) => {
						eprintln!("The path {} does not exist or cannot be accessed. Error: {}", path.display(), e);
						None
					}
				}
			})
			.collect::<HashSet<PathBuf>>()
			.into_iter()
			.collect();
	let recursive = matches.get_flag("recursive");
	let remove_fill = matches.get_flag("remove-fill");
	let use_svgo = matches.get_flag("svgo");
	let compress_svgz = matches.get_flag("svgz");
	let no_default = matches.get_flag("no-default");
	let quiet = matches.get_flag("quiet");

	if no_default && !use_svgo && !compress_svgz {
		if !quiet {
			println!("No actions specified. Your files were not modified.");
			println!("Type 'svgc --help' for more information.");
		}
		return ExitCode::SUCCESS
	}

	let svgo_path = if use_svgo {
		match which::which("svgo") {
			Ok(path) => Some(path.display().to_string()),
			Err(_) => {
				eprintln!("Error: SVGO is not installed.");
				if !quiet { println!("Your files weren't modified."); }
				return ExitCode::FAILURE
			}
		}
	} else {
		None
	};

	if paths.is_empty() {
		return ExitCode::SUCCESS
	}

	let svg_files = match files::find_svg_files(&paths, recursive) {
		Ok(files) => files,
		Err(e) => {
			eprintln!("Error finding svg files: {e}");
			if !quiet { println!("Your files weren't modified."); }
			return ExitCode::FAILURE
		}
	};

	let mut backup_storage = match TempBackupStorage::new(&svg_files) {
		Ok(storage) => storage,
		Err(e) => {
			eprintln!("Error creating temporary backup storage: {e}");
			if !quiet { println!("Your files weren't modified."); }
			return ExitCode::FAILURE
		}
	};

	backup_storage.disable_auto_cleanup();

	if !no_default {
		if let Err(e) = default_optimize_files(&svg_files, remove_fill) {
			eprintln!("Error optimizing files: {e}");
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		}
	}

	if use_svgo && svgo_path != None {
		if let Err(e) = run_svgo(&svg_files, &svgo_path.unwrap()) {
			eprintln!("Error optimizing files with SVGO: {e}");
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		};
	}

	if compress_svgz {
		if let Err(e) = compress_to_svgz(&svg_files) {
			eprintln!("Error compressing files to .svgz format: {e}");
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		}
	}

	backup_storage.enable_auto_cleanup();

	if !quiet {
		println!("Files were successfully compressed.");
	}

	ExitCode::SUCCESS
}

fn try_to_copy_back(temp_storage: &mut TempBackupStorage, quiet: bool) {
	if let Err(e) = temp_storage.copy_back() {
		temp_storage.disable_auto_cleanup();
		eprintln!("Error restoring files: {}.\nBackups are located in {} directory.", e, temp_storage.temp_dir().display());
		return
	} else {
		temp_storage.enable_auto_cleanup();
	}
	if !quiet {
		println!("Your files were restored.");
	}
}
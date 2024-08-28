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
use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Arg, ArgAction, ArgAction::SetTrue, Command};
use lazy_static::lazy_static;
use rust_i18n::{i18n, t};

mod default_opt;
mod files;
mod svgo;
mod svgz;
mod i18n;

use default_opt::default_optimize_files;
use files::TempBackupStorage;
use svgo::run_svgo;
use svgz::compress_to_svgz;
use i18n::set_rust_i18n_locale;

i18n!();

fn main() -> ExitCode {
	set_rust_i18n_locale();

	lazy_static! { // need static variables for clap
		static ref about            : Cow<'static, str> = t!("about");
		static ref version          : Cow<'static, str> = t!("version");
		static ref long_version     : Cow<'static, str> = t!("long-version");
		static ref paths_help       : Cow<'static, str> = t!("paths-help");
		static ref paths_value_name : Cow<'static, str> = t!("paths-value-name");
	    static ref recursive_help   : Cow<'static, str> = t!("recursive-help");
	    static ref remove_fill_help : Cow<'static, str> = t!("remove-fill-help");
	    static ref svgo_help        : Cow<'static, str> = t!("svgo-help");
	    static ref svgz_help        : Cow<'static, str> = t!("svgz-help");
	    static ref no_default_help  : Cow<'static, str> = t!("no-default-help");
	    static ref quiet_help       : Cow<'static, str> = t!("quiet-help");
	    static ref version_help     : Cow<'static, str> = t!("version-help");
	    static ref help_help        : Cow<'static, str> = t!("help-help");
	}

	let matches = Command::new("svgc")
		.about(&about[..])
		.version(&version[..])
		.long_version(&long_version[..])
		.arg(Arg::new("paths").help(&paths_help[..])
			.value_name(&paths_value_name[..])
			.required(true)
			.num_args(1..))
		.arg(Arg::new("recursive")  .short('r').long("recursive")  .help(&recursive_help[..])  .action(SetTrue))
		.arg(Arg::new("remove-fill").short('f').long("remove-fill").help(&remove_fill_help[..]).action(SetTrue))
		.arg(Arg::new("svgo")       .short('o').long("svgo")       .help(&svgo_help[..])       .action(SetTrue))
		.arg(Arg::new("svgz")       .short('z').long("svgz")       .help(&svgz_help[..])       .action(SetTrue))
		.arg(Arg::new("no-default") .short('n').long("no-default") .help(&no_default_help[..]) .action(SetTrue))
		.arg(Arg::new("quiet")      .short('q').long("quiet")      .help(&quiet_help[..])      .action(SetTrue))
		.disable_version_flag(true)
		.arg(Arg::new("version")    .short('v').long("version")    .help(&version_help[..])    .action(ArgAction::Version))
		.disable_help_flag(true)
		.arg(Arg::new("help")       .short('h').long("help")       .help(&help_help[..])       .action(ArgAction::Help))
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
							eprintln!("{}", t!("path-not-svg-or-dir", path = path.display()));
							None
						}
					}
					Err(e) => {
						eprintln!("{}", t!("path-not-exist-or-no-access", path = path.display(), error = e));
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
			println!("{}", t!("no-action-specified-files-not-modified"));
			println!("{}", t!("type-svg-help-for-more-information"));
		}
		return ExitCode::SUCCESS
	}

	let svgo_path = if use_svgo {
		match which::which("svgo") {
			Ok(path) => Some(path),
			Err(_) => {
				eprintln!("{}", t!("error-svgo"));
				if !quiet { println!("{}", t!("your-files-were-not-modified")); }
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
			eprintln!("{}", t!("error-finding-svg-files", error = e));
			if !quiet { println!("{}", t!("your-files-were-not-modified")); }
			return ExitCode::FAILURE
		}
	};

	let mut backup_storage = match TempBackupStorage::new(&svg_files) {
		Ok(storage) => storage,
		Err(e) => {
			eprintln!("{}", t!("error-creating-temporary-backup-storage", error = e));
			if !quiet { println!("{}", t!("your-files-were-not-modified")); }
			return ExitCode::FAILURE
		}
	};

	backup_storage.disable_auto_cleanup();

	if !no_default {
		if let Err(e) = default_optimize_files(&svg_files, remove_fill) {
			eprintln!("{}", t!("error-optimizing-files", error = e));
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		}
	}

	if use_svgo && svgo_path != None {
		if let Err(e) = run_svgo(&svg_files, &svgo_path.unwrap()) {
			eprintln!("{}", t!("error-optimizing-files-with-svgo", error = e));
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		};
	}

	if compress_svgz {
		if let Err(e) = compress_to_svgz(&svg_files) {
			eprintln!("{}", t!("error-compressing-files", error = e));
			try_to_copy_back(&mut backup_storage, quiet);
			return ExitCode::FAILURE
		}
	}

	backup_storage.enable_auto_cleanup();

	if !quiet {
		println!("{}", t!("files-successfully-compressed"));
	}

	ExitCode::SUCCESS
}

fn try_to_copy_back(temp_storage: &mut TempBackupStorage, quiet: bool) {
	if let Err(e) = temp_storage.copy_back() {
		temp_storage.disable_auto_cleanup();
		eprintln!("{}", t!("error-restoring-files", error = e, dir = temp_storage.temp_dir().display()));
		return
	} else {
		temp_storage.enable_auto_cleanup();
	}
	if !quiet {
		println!("{}", t!("files-restored"));
	}
}
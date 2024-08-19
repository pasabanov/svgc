//! © 2024 Petr Alexandrovich Sabanov. This code is licensed under the Creative Commons Attribution 4.0 International License (CC BY 4.0).
//!
//! You are free to:
//! - Share — copy and redistribute the material in any medium or format
//! - Adapt — remix, transform, and build upon the material
//!
//! You must give appropriate credit, provide a link to the license, and indicate if changes were made. You may do so in any reasonable manner, but not in any way that suggests the licensor endorses you or your use.
//!
//! For full license details, see https://creativecommons.org/licenses/by/4.0/

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::io::Read;

use clap::Arg;
use clap::ArgAction::SetTrue;
use flate2::Compression;
use flate2::write::GzEncoder;
use lazy_static::lazy_static;
use regex::Regex;

fn default_optimize(filepath: &Path, remove_fill: bool) -> io::Result<()> {
	let mut content = fs::read_to_string(filepath)?;

	// Define regular expressions
	lazy_static! {
		static ref R_FILL: Regex = Regex::new(r#"fill="[^"]*""#).unwrap();
		static ref R_XLINK_HREF: Regex = Regex::new(r#"xlink:href"#).unwrap();
		static ref R_XMLNS_XLINK: Regex = Regex::new(r#"\s+xmlns:xlink="[^"]*\""#).unwrap();
		static ref R_COMMENT: Regex = Regex::new(r#"<!--.*?-->"#).unwrap();
		static ref R_XML_TAG: Regex = Regex::new(r#"<\?xml.*?>"#).unwrap();
		static ref R_DOCTYPE_SVG: Regex = Regex::new(r#"<!DOCTYPE svg[^>]*>"#).unwrap();
		static ref R_WHITESPACE: Regex = Regex::new(r#"\s+"#).unwrap();
		static ref R_WHITESPACE_AROUND_TAGS: Regex = Regex::new(r#"\s*([<>])\s*"#).unwrap();
		static ref R_SYMBOLS_BETWEEN_TAGS: Regex = Regex::new(r#">[^<]+<"#).unwrap();
		static ref R_XML_SPACE: Regex = Regex::new(r#"\s+xml:space="[^"]+""#).unwrap();
	}

	content = content.trim().to_string();
	if remove_fill {
		content = R_FILL.replace_all(&content, "").to_string();
	}
	if R_XLINK_HREF.find(&content).is_none() {
		content = R_XMLNS_XLINK.replace_all(&content, "").to_string();
	}
	content = R_COMMENT.replace_all(&content, "").to_string();
	content = R_XML_TAG.replace_all(&content, "").to_string();
	content = R_DOCTYPE_SVG.replace_all(&content, "").to_string();
	content = R_WHITESPACE.replace_all(&content, " ").to_string();
	content = R_WHITESPACE_AROUND_TAGS.replace_all(&content, "$1").to_string();
	if R_SYMBOLS_BETWEEN_TAGS.find(&content).is_none() {
		content = R_XML_SPACE.replace_all(&content, "").to_string();
	}

	fs::write(filepath, content)
}

fn compress_to_svgz(filepath: &Path) -> io::Result<()> {
	let svgz_filepath = format!("{}z", filepath.display());
	let file = fs::File::open(filepath)?;
	let reader = io::BufReader::new(file);

	let file = fs::File::create(&svgz_filepath)?;
	let mut encoder = GzEncoder::new(file, Compression::best());

	// Copy contents from reader to encoder
	io::copy(&mut reader.take(u64::MAX), &mut encoder)?;

	encoder.finish()?;
	fs::remove_file(filepath)?;
	Ok(())
}

fn find_svg_files(vec_to_append: &mut Vec<PathBuf>, path: &PathBuf, recursive: bool) -> io::Result<()> {
	if path.is_file() {
		if path.extension().and_then(|e| e.to_str()) == Some("svg") {
			vec_to_append.push(path.clone());
		}
		return Ok(());
	}
	for entry in fs::read_dir(path)? {
		let entry = entry?;
		let path = entry.path();
		if path.is_dir() && recursive {
			find_svg_files(vec_to_append, &path, recursive)?;
		} else if path.extension().and_then(|e| e.to_str()) == Some("svg") {
			vec_to_append.push(path.clone());
		}
	}
	Ok(())
}

fn main() -> io::Result<()> {
	let matches = clap::Command::new("SVG Compressor")
		.version("0.1.0")
		.about("Compress SVG files by removing unnecessary whitespace, comments, metadata, and some other data.")
		.arg(Arg::new("paths").help("List of SVG files or directories or SVG files to compress.")
			.required(true)
			.num_args(1..))
		.arg(Arg::new("recursive")  .short('r').long("recursive")  .help("Recursively process directories.")
			.action(SetTrue))
		.arg(Arg::new("remove-fill").short('f').long("remove-fill").help("Remove fill=\"...\" attributes.")
			.action(SetTrue))
		.arg(Arg::new("svgo")       .short('o').long("svgo")       .help("Use svgo if it exists in the system.")
			.action(SetTrue))
		.arg(Arg::new("svgz")       .short('z').long("svgz")       .help("Compress to .svgz format with gzip utility after processing.")
			.action(SetTrue))
		.arg(Arg::new("no-default") .short('n').long("no-default") .help("Skip default optimizations.")
			.action(SetTrue))
		.get_matches();

	let paths: Vec<PathBuf> = matches.get_many::<String>("paths").unwrap().map(PathBuf::from).collect();
	let recursive = matches.get_flag("recursive");
	let remove_fill = matches.get_flag("remove-fill");
	let use_svgo = matches.get_flag("svgo");
	let compress_svgz = matches.get_flag("svgz");
	let no_default = matches.get_flag("no-default");

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

	if !no_default {
		for file in &svg_files {
			default_optimize(file, remove_fill)?;
		}
	}

	if use_svgo && svgo_path != None {
		let mut command = process::Command::new(&svgo_path.unwrap());
		command.args(&["-q"]).args(&svg_files);
		// Two times for additional optimization
		if !command.status()?.success() || !command.status()?.success() {
			eprintln!("Error during SVGO optimization.");
		}
	}

	if compress_svgz {
		for file in &svg_files {
			compress_to_svgz(file)?;
		}
	}

	Ok(())
}
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

use std::fs;
use std::io;
use std::path::Path;
use regex::Regex;
use lazy_static::lazy_static;

pub fn default_optimize(filepath: &Path, remove_fill: bool) -> io::Result<()> {
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
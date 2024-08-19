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
use std::path::PathBuf;

pub(crate) fn find_svg_files(vec_to_append: &mut Vec<PathBuf>, path: &PathBuf, recursive: bool) -> io::Result<()> {
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
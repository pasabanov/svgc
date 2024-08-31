//! svgc is a tool for compressing SVG files
//! Copyright (C) © 2024  Petr Alexandrovich Sabanov
//!
//! This program is free software: you can redistribute it and/or modify
//! it under the terms of the GNU Affero General Public License as published by
//! the Free Software Foundation, either version 3 of the License, or
//! (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful,
//! but WITHOUT ANY WARRANTY; without even the implied warranty of
//! MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
//! GNU Affero General Public License for more details.
//!
//! You should have received a copy of the GNU Affero General Public License
//! along with this program.  If not, see <https://www.gnu.org/licenses/>.

use std::io;
use std::path::PathBuf;
use std::process;

pub fn run_svgo(svg_files: &Vec<PathBuf>, svgo_path: &PathBuf) -> io::Result<()> {
	let mut command = process::Command::new(svgo_path);
	command.args(&["-q"]).args(svg_files);
	command.status()?;
	command.status()?; // Second time for additional optimization
	Ok(())
}
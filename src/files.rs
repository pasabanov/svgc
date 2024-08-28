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

use std::{env, fs, io};
use std::collections::HashSet;
use std::path::PathBuf;
use chrono::Local;

fn unique_timestamp() -> String {
	Local::now().format("%Y-%m-%d_%H-%M-%S_%f").to_string()
}

fn generate_temp_dir_name() -> String {
	format!("svgc_temp_files_{}", unique_timestamp())
}

fn try_create_temp_dir(path: &PathBuf, name: &str) -> io::Result<PathBuf> {
	let temp_dir = path.join(name);
	fs::create_dir_all(&temp_dir)?;
	Ok(temp_dir)
}

pub fn create_temp_dir() -> Option<PathBuf> {
	let temp_dir_name = generate_temp_dir_name();

	// Trying to generate temporary directory in some of these directories
	let directories = [
		|| { // current directory, checking that is not temporary
			let current = fs::canonicalize(env::current_dir().ok()?).ok()?;
			let temp = fs::canonicalize(env::temp_dir()).ok()?;
			if current != temp { Some(current) } else { None }
		},
		|| dirs::home_dir(),
		|| directories::ProjectDirs::from("org", "pasabanov", "svgc").map(|dirs| dirs.data_dir().to_path_buf()),
	];

	for (i, get_dir) in directories.iter().enumerate() {
		if let Some(dir) = get_dir() {
			if let Ok(temp_dir) = try_create_temp_dir(&dir, &temp_dir_name) {
				return Some(temp_dir);
			}
			eprintln!(
				"Couldn't create a temporary directory in {}.{}",
				dir.display(),
				if i < directories.len() - 1 { " Trying next." } else { "" }
			);
		}
	}

	None
}

pub struct TempBackupStorage {
	paths: Vec<(PathBuf, PathBuf)>,
	temp_dir: PathBuf,
	auto_cleanup: bool,
}

#[allow(dead_code)]
impl TempBackupStorage {

	pub fn new(original_paths: &[PathBuf]) -> io::Result<Self> {

		let temp_dir = create_temp_dir()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Could not create temporary directory"))?;

		let mut paths = Vec::new();

		for path in original_paths {
			let temp_path = temp_dir.join(
				format!(
					"{}_{}.{}",
					path.file_stem()
						.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Could not get file {} name", path.display())))?
						.to_string_lossy(),
					unique_timestamp(),
					path.extension()
						.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, format!("Could not get file {} extension", path.display())))?
						.to_string_lossy(),
				)
			);
			if path.is_file() {
				fs::copy(path, &temp_path)?;
			} else if path.is_dir() {
				copy_dir(path, &temp_path)?;
			}
			paths.push((path.clone(), temp_path));
		}

		Ok(Self { paths, temp_dir, auto_cleanup: true, })
	}

	pub fn copy_back(&self) -> io::Result<()> {
		for (orig_path, temp_bak_path) in self.paths.iter() {
			if let Some(parent) = temp_bak_path.parent() {
				fs::create_dir_all(parent)?;
			} else {
				eprintln!("Warning! Could not get parent directory of file {}", temp_bak_path.display());
			}
			if temp_bak_path.is_file() {
				fs::copy(temp_bak_path, orig_path)?;
			} else if temp_bak_path.is_dir() {
				copy_dir(temp_bak_path, orig_path)?;
			}
		}
		Ok(())
	}

	fn orig_paths_iter(&self) -> impl Iterator<Item = &PathBuf> {
		self.paths.iter().map(|(orig, _)| orig)
	}

	fn temp_paths_iter(&self) -> impl Iterator<Item = &PathBuf> {
		self.paths.iter().map(|(_, temp)| temp)
	}

	pub fn temp_dir(&self) -> &PathBuf {
		&self.temp_dir
	}

	pub fn is_auto_cleanup(&self) -> bool {
		self.auto_cleanup
	}

	pub fn disable_auto_cleanup(&mut self) {
		self.auto_cleanup = false
	}

	pub fn enable_auto_cleanup(&mut self) {
		self.auto_cleanup = true
	}

	pub fn set_auto_cleanup(&mut self, auto_cleanup: bool) {
		self.auto_cleanup = auto_cleanup
	}

	pub fn cleanup(&mut self) -> io::Result<()> {
		fs::remove_dir_all(&self.temp_dir)
	}
}

impl Drop for TempBackupStorage {
	fn drop(&mut self) {
		if self.auto_cleanup {
			if let Err(e) = self.cleanup() {
				eprintln!("Failed to delete temporary directory {}: {}", self.temp_dir.display(), e);
			}
		}
	}
}

fn copy_dir(src: &PathBuf, dst: &PathBuf) -> io::Result<()> {
	fs::create_dir_all(&dst)?;
	for entry in fs::read_dir(src)? {
		let entry = entry?;
		let src_path = entry.path();
		let dst_path = dst.join(entry.file_name());
		if src_path.is_file() {
			fs::copy(src_path, dst_path)?;
		} else if src_path.is_dir() {
			copy_dir(&src_path, &dst_path)?;
		}
	}
	Ok(())
}

pub fn find_svg_files(paths: &[PathBuf], recursive: bool) -> io::Result<Vec<PathBuf>> {

	fn find_append_svg_files(container: &mut HashSet<PathBuf>, path: &PathBuf, recursive: bool) -> io::Result<()> {
		if path.is_file() {
			if path.extension().and_then(|e| e.to_str()) == Some("svg") {
				container.insert(path.clone());
			}
			return Ok(())
		} else if !path.is_dir() {
			return Ok(())
		}
		for entry in fs::read_dir(path)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_file() || recursive && path.is_dir() {
				find_append_svg_files(container, &path, recursive)?;
			}
		}
		Ok(())
	}

	let mut svg_files = HashSet::new();
	for temp_path in paths {
		find_append_svg_files(&mut svg_files, &temp_path, recursive)?;
	}

	let svg_files = svg_files.into_iter().collect();

	Ok(svg_files)
}
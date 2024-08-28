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
use rust_i18n::t;

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
			eprintln!("{}",
				t!("could-not-create-temp-dir-in-dir",
					dir = dir.display(),
					suffix = if i < directories.len() - 1 { " Trying next." } else { "" }
				)
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
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, t!("could-not-create-temporary-directory")))?;

		fn initialize_paths(original_paths: &[PathBuf], temp_dir: &PathBuf) -> io::Result<Vec<(PathBuf, PathBuf)>> {
			let mut paths = Vec::new();
			for path in original_paths {
				let temp_path = temp_dir.join(
					format!(
						"{}_{}.{}",
						path.file_stem()
							.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, t!("could-not-get-file-name", path = path.display())))?
							.to_string_lossy(),
						unique_timestamp(),
						path.extension()
							.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, t!("could-not-get-file-ext", path = path.display())))?
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
			Ok(paths)
		}

		match initialize_paths(original_paths, &temp_dir) {
			Ok(paths) => {
				Ok(Self { paths, temp_dir, auto_cleanup: true, })
			}
			Err(e) => {
				if let Err(cleanup_error) = fs::remove_dir_all(&temp_dir) {
					eprintln!("{}", t!("failed-to-delete-temp-dir", dir = temp_dir.display(), error = cleanup_error));
				}
				Err(e)
			}
		}
	}

	pub fn copy_back(&self) -> io::Result<()> {
		for (orig_path, temp_bak_path) in self.paths.iter() {
			if let Some(parent) = temp_bak_path.parent() {
				fs::create_dir_all(parent)?;
			} else {
				eprintln!("{}", t!("warning-could-not-get-parent-dir", path = temp_bak_path.display()));
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
		if self.temp_dir.try_exists()? {
			fs::remove_dir_all(&self.temp_dir)?;
		}
		Ok(())
	}
}

impl Drop for TempBackupStorage {
	fn drop(&mut self) {
		if self.auto_cleanup {
			if let Err(e) = self.cleanup() {
				eprintln!("{}", t!("failed-to-delete-temp-dir", dir = self.temp_dir.display(), error = e));
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unique_timestamp() {
		assert_ne!(unique_timestamp(), unique_timestamp());
	}

	#[test]
	fn test_generate_temp_dir_name() {
		assert_ne!(generate_temp_dir_name(), generate_temp_dir_name());
	}
	
	#[test]
	#[allow(non_snake_case)]
	fn test_TempBackupStorage() {
		let temp_backup_storage = TempBackupStorage::new(&vec![]);
		assert!(temp_backup_storage.is_ok());
		let mut temp_backup_storage = temp_backup_storage.unwrap();
		assert!(temp_backup_storage.temp_dir().exists());
		assert!(temp_backup_storage.cleanup().is_ok());
		assert!(!temp_backup_storage.temp_dir().exists());

		let temp_backup_storage = TempBackupStorage::new(&vec![]);
		assert!(temp_backup_storage.is_ok());
		let temp_backup_storage = temp_backup_storage.unwrap();
		let temp_dir = temp_backup_storage.temp_dir().clone();
		assert!(temp_dir.exists());
		drop(temp_backup_storage);
		assert!(!temp_dir.exists());
	}
}
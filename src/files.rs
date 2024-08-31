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

use std::{env, fs};
use std::io::{self, IsTerminal};
use std::path::{Path, PathBuf};

use chrono::Local;
use rust_i18n::t;

use crate::default_opt::default_optimize;
use crate::svgo::run_svgo;
use crate::svgz::compress_to_svgz;

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

pub fn is_svg_file(path: &Path) -> bool {
	path.is_file() && (path.extension() == Some("svg".as_ref()) || path.file_name() == Some(".svg".as_ref()))
}

struct SvgFile {
	original_path: PathBuf,
	backup_path: PathBuf,
	result_path: Option<PathBuf>,
	original_size: u64,
	result_size: Option<u64>,
}

#[allow(dead_code)]
impl SvgFile {
	pub fn new(original_path: PathBuf, backup_dir: &Path) -> io::Result<Self> {
		if is_svg_file(&original_path) {
			let original_size = original_path.metadata()?.len();
			let backup_path = backup_dir.join(
				format!(
					"{}_{}.{}",
					original_path.file_stem()
						.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, t!("could-not-get-file-name", path = original_path.display())))?
						.to_string_lossy(),
					unique_timestamp(),
					original_path.extension()
						.unwrap_or_default()
						.to_string_lossy(),
				)
			);
			fs::copy(&original_path, &backup_path)?;
			Ok(Self {
				original_path,
				backup_path,
				result_path: None,
				original_size,
				result_size: None,
			})
		} else {
			Err(io::Error::new(io::ErrorKind::NotFound, t!("path-not-svg", path = original_path.display())))
		}
	}

	pub fn apply_default_optimizations(&self, remove_fill: bool) -> io::Result<()> {
		default_optimize(&self.original_path, remove_fill)
	}

	pub fn compress(&mut self) -> io::Result<()> {
		self.result_path = Some(compress_to_svgz(&self.original_path)?);
		Ok(())
	}

	pub fn calculate_result_size(&mut self) -> io::Result<()> {
		if self.result_size != None {
			return Ok(())
		}
		let path = self.result_path.as_deref().unwrap_or(&self.original_path);
		self.result_size = Some(path.metadata()?.len());
		Ok(())
	}

	pub fn restore(&self) -> io::Result<()> {
		fs::copy(&self.backup_path, &self.original_path).map(|_| ())
	}

	pub fn original_path(&self) -> &Path {
		&self.original_path
	}

	pub fn backup_path(&self) -> &Path {
		&self.backup_path
	}

	pub fn result_path(&self) -> Option<&Path> {
		self.result_path.as_deref()
	}

	pub fn original_size(&self) -> u64 {
		self.original_size
	}

	pub fn result_size(&self) -> Option<u64> {
		self.result_size
	}
}

pub struct SvgFileGroup {
	files: Vec<SvgFile>,
	backup_dir: PathBuf,
	auto_delete_backups: bool,
}

#[allow(dead_code)]
impl SvgFileGroup {
	pub fn new(paths: Vec<PathBuf>, auto_delete_backups: bool) -> io::Result<Self> {
		let backup_dir = create_temp_dir()
			.ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, t!("could-not-create-temporary-directory")))?;
		fn initialize_files(paths: Vec<PathBuf>, backup_dir: &Path) -> io::Result<Vec<SvgFile>> {
			paths.into_iter().map(|path| SvgFile::new(path, backup_dir)).collect()
		}
		match initialize_files(paths, &backup_dir) {
			Ok(files) => Ok(Self {files, backup_dir, auto_delete_backups}),
			Err(e) => {
				if let Err(cleanup_error) = fs::remove_dir_all(&backup_dir) {
					eprintln!("{}", t!("failed-to-delete-temp-dir", dir = backup_dir.display(), error = cleanup_error));
				}
				Err(e)
			}
		}
	}

	pub fn apply_default_optimizations(&self, remove_fill: bool) -> io::Result<()> {
		for file in &self.files {
			file.apply_default_optimizations(remove_fill)?
		}
		Ok(())
	}

	pub fn apply_svgo(&self, svgo_path: &Path) -> io::Result<()> {
		run_svgo(self.files.iter().map(|f| f.original_path.as_path()), svgo_path)
	}

	pub fn compress(&mut self) -> io::Result<()> {
		for file in &mut self.files {
			file.compress()?
		}
		Ok(())
	}

	pub fn print_summary(&mut self) -> io::Result<()> {

		let mut total_before: u64 = 0;
		let mut total_after: u64 = 0;

		let current_dir = env::current_dir().ok();

		for file in &mut self.files {
			file.calculate_result_size()?;

			let original_size = file.original_size();
			let result_size = file.result_size().unwrap();

			total_before += original_size;
			total_after += result_size;

			let size_diff = original_size.saturating_sub(result_size);
			let size_diff_percent = (size_diff as f64 / original_size as f64) * 100.0;

			let original_path = file.original_path();
			let result_path = file.result_path().unwrap_or(original_path);

			let (relative_file, relative_final_path) = if let Some(ref dir) = current_dir {
				(original_path.strip_prefix(dir).unwrap_or(original_path), result_path.strip_prefix(dir).unwrap_or(&result_path))
			} else {
				(original_path, result_path)
			};

			let file_name_display = if relative_final_path != relative_file {
				format!("{} -> {}", relative_file.display(), relative_final_path.display())
			} else {
				relative_file.display().to_string()
			};

			let percent_str = if size_diff_percent > 0.0 && io::stdout().is_terminal() {
				format!("\x1b[32m{:.2}%\x1b[0m", size_diff_percent) // Green
			} else {
				format!("{:.2}%", size_diff_percent)
			};

			println!("{file_name_display}:\n{original_size} - {percent_str} = {result_size} {}\n", t!("bytes"));
		}

		let total_diff = total_before.saturating_sub(total_after);
		let total_diff_percent = (total_diff as f64 / total_before as f64) * 100.0;

		let total_str = t!("total");
		let bytes_str = t!("bytes");

		println!("{total_str}: {total_before} -> {total_after} {bytes_str} (-{total_diff} {bytes_str}, -{:.2}%)", total_diff_percent);

		Ok(())
	}

	pub fn restore_files(&self) -> io::Result<()> {
		for file in &self.files {
			file.restore()?;
		}
		Ok(())
	}

	pub fn backup_dir(&self) -> &Path {
		&self.backup_dir
	}

	pub fn is_auto_delete_backups(&self) -> bool {
		self.auto_delete_backups
	}

	pub fn enable_auto_delete_backups(&mut self) {
		self.auto_delete_backups = true;
	}

	pub fn disable_auto_delete_backups(&mut self) {
		self.auto_delete_backups = false;
	}

	pub fn delete_backups(&mut self) -> io::Result<()> {
		if self.backup_dir.try_exists()? {
			fs::remove_dir_all(&self.backup_dir)?;
		}
		Ok(())
	}
}

impl Drop for SvgFileGroup {
	fn drop(&mut self) {
		if self.auto_delete_backups {
			if let Err(e) = self.delete_backups() {
				eprintln!("{}", t!("failed-to-delete-backups-dir", dir = self.backup_dir.display(), error = e));
			}
		}
	}
}

pub fn find_svg_files(paths: &[PathBuf], recursive: bool) -> io::Result<Vec<PathBuf>> {

	fn find_append_svg_files(container: &mut Vec<PathBuf>, path: &PathBuf, recursive: bool) -> io::Result<()> {
		if path.is_file() {
			if path.extension().and_then(|e| e.to_str()) == Some("svg") {
				container.push(path.clone());
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

	let mut svg_files = Vec::new();
	for temp_path in paths {
		find_append_svg_files(&mut svg_files, &temp_path, recursive)?;
	}
	svg_files.sort();
	svg_files.dedup();

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
	fn test_SvgFileGroup() {
		let svg_file_group = SvgFileGroup::new(vec![], true);
		assert!(svg_file_group.is_ok());
		let mut svg_file_group = svg_file_group.unwrap();
		assert!(svg_file_group.backup_dir().exists());
		assert!(svg_file_group.delete_backups().is_ok());
		assert!(!svg_file_group.backup_dir().exists());

		let svg_file_group = SvgFileGroup::new(vec![], true);
		assert!(svg_file_group.is_ok());
		let svg_file_group = svg_file_group.unwrap();
		let backup_dir = svg_file_group.backup_dir().to_path_buf();
		assert!(backup_dir.exists());
		drop(svg_file_group);
		assert!(!backup_dir.exists());
	}
}
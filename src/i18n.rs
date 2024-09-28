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

/// The default locale used as a fallback when the desired locale is not found.
///
/// Set to `"en"` to ensure maximum universality.
/// This choice avoids issues with unsupported regions, variants, etc.
const DEFAULT_LOCALE: &str = "en";

/// Sets the locale for the `rust_i18n` library.
///
/// This function sets the locale used by the `rust_i18n` library to the best matching locale from
/// the list of available locales, based on the system locales.
pub fn set_rust_i18n_locale() {
	rust_i18n::set_locale(locale_match::bcp47::best_matching_locale(rust_i18n::available_locales!(), sys_locale::get_locales()).unwrap_or(DEFAULT_LOCALE));
}
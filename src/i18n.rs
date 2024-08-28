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

use language_tags::LanguageTag;

/// The default locale used as a fallback when the desired locale is not found.
///
/// Set to `"en"` to ensure maximum universality.
/// This choice avoids issues with unsupported regions, variants, etc.
const DEFAULT_LOCALE: &str = "en";

/// Finds the best matching locale from a list of available locales based on a list of user locales.
///
/// The function compares user locales to available locales to find the best match.  
/// For each user locale, it iterates through the available locales and, for those with a matching
/// primary language, calculates a score based on how closely each available locale matches the user
/// locale.  
/// The score calculation gives higher priority to matching more significant parts of the locale
/// (i.e., earlier segments in the locale string). If a subtag is empty, it is considered to match
/// equally well with any subtag from the same category.
///
/// If multiple available locales have the same score, the function selects the one that appears
/// earlier in the list of available locales.  
/// If no available locale matches the primary language of a user locale, the function moves to the
/// next user locale in the list.  
/// If no matches are found for any user locale, the function returns [`None`].
///
/// Malformed locales are ignored.
///
/// # Arguments
///
/// * `available_locales` - An iterator over locale strings representing the available locales.
///   These locales should be ordered by priority, meaning that a locale appearing earlier in this
///   list is considered more preferable for the program.
/// * `user_locales` - An iterator over locale strings representing the user locales to match
///   against. These locales should also be ordered by priority, meaning that a locale appearing
///   earlier in this list is considered more desirable for the user.
///
/// # Returns
///
/// Returns an [`Option<String>`] containing the string representation of the best matching locale.  
/// If multiple available locales match the same user locale with equal score, the one that appears
/// earlier in the list of available locales is chosen.  
/// If no match is found, [`None`] is returned.
///
/// # Examples
///
/// ```
/// let available_locales = vec!["en-US", "en-GB", "ru-UA", "fr-FR", "it"];
/// let user_locales = vec!["ru-RU", "ru", "en-US", "en"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // "ru-UA" is the best match for the highest-priority user locale "ru-RU"
/// assert_eq!(best_match, Some("ru-UA".to_string()));
/// ```
///
/// ```
/// let available_locales = vec!["en", "pt-BR", "pt-PT", "es"];
/// let user_locales = vec!["pt", "en"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // "pt-BR" is the first best match for the highest-priority user locale "pt"
/// assert_eq!(best_match, Some("pt-BR".to_string()));
/// ```
///
/// ```
/// let available_locales = vec!["zh", "zh-cmn", "zh-cmn-Hans"];
/// let user_locales = vec!["zh-Hans"];
///
/// let best_match = best_matching_locale(available_locales.iter(), user_locales.iter());
///
/// // Empty extended language subtag in "zh-Hans" matches any extended language, e.g. "cmn"
/// assert_eq!(best_match, Some("zh-cmn-Hans".to_string()));
/// ```
///
fn best_matching_locale<T1, T2>(available_locales: impl Iterator<Item = T1>, user_locales: impl Iterator<Item = T2>) -> Option<String>
where
	T1: AsRef<str>,
	T2: AsRef<str>
{
	let available_tags = available_locales
		.filter_map(|l| LanguageTag::parse(l.as_ref()).ok())
		.collect::<Vec<LanguageTag>>();

	user_locales
		.filter_map(|locale| LanguageTag::parse(locale.as_ref()).ok())
		.find_map(|user_tag|
			available_tags.iter()
				.rev() // For max_by_key to return the first tag with max score
				.filter(|aval_tag| aval_tag.primary_language() == user_tag.primary_language())
				.max_by_key(|aval_tag| {
					let mut score = 0;
					for (aval, user, weight) in [
						(aval_tag.extended_language(), user_tag.extended_language(), 32),
						(aval_tag.script(),            user_tag.script(),            16),
						(aval_tag.region(),            user_tag.region(),             8),
						(aval_tag.variant(),           user_tag.variant(),            4),
						// TODO: Implement separate comparison for each extension
						(aval_tag.extension(),         user_tag.extension(),          2),
						(aval_tag.private_use(),       user_tag.private_use(),        1),
					] {
						match (aval, user) {
							(Some(a), Some(u)) if a == u => score += weight,
							_ => {} // Ignore if both are None
						}
					}
					score
				})
		)
		.map(|aval_tag| aval_tag.to_string())
}

/// Sets the locale for the `rust_i18n` library.
///
/// This function sets the locale used by the `rust_i18n` library to the best matching locale
/// from the list of available locales, based on the system locale.
pub fn set_rust_i18n_locale() {
	rust_i18n::set_locale(&best_matching_locale(rust_i18n::available_locales!().iter(), sys_locale::get_locales()).unwrap_or(DEFAULT_LOCALE.to_string()));
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_best_matching_locale() {

		fn check_best_match(available_locales: &[&str], user_locales: &[&str], expected: Option<&str>) -> bool {
			best_matching_locale(available_locales.iter(), user_locales.iter()).as_deref() == expected
		}

		// One best match
		assert!(check_best_match(&["en-US", "ru-RU"], &["ru", "en"], Some("ru-RU")));
		assert!(check_best_match(&["en-US", "ru-RU"], &["en", "ru"], Some("en-US")));
		assert!(check_best_match(&["en-US", "en-GB", "ru-UA", "fr-FR", "it"], &["ru-RU", "ru", "en-US", "en"], Some("ru-UA")));

		// Multiple best matches
		assert!(check_best_match(&["en-US", "en-GB", "ru-UA", "fr-FR", "it"], &["en-US", "en", "ru-RU", "ru"], Some("en-US")));
		assert!(check_best_match(&["en", "pt-BR", "pt-PT", "es"], &["pt", "en"], Some("pt-BR")));

		// One available locale
		assert!(check_best_match(&["kk"], &["en", "en-US", "fr-FR", "fr", "it", "pt", "ru-RU", "es-ES", "kk-KZ"], Some("kk")));

		// One user locale
		assert!(check_best_match(&["en", "en-US", "fr-FR", "fr", "it", "pt", "ru-RU", "es-ES", "kk-KZ", "pt"], &["pt-PT"], Some("pt")));

		// Not found
		assert!(check_best_match(&["en", "en-US", "fr-FR", "fr", "it", "pt", "es-ES", "kk-KZ", "pt"], &["ru"], None));
		assert!(check_best_match(&["en", "en-US", "fr-FR", "fr", "pt"], &["id"], None));
		assert!(check_best_match(&["ru", "be", "uk", "kk"], &["en"], None));

		// Empty available locales
		assert!(check_best_match(&[], &["en", "fr", "it", "pt"], None));

		// Empty user locales
		assert!(check_best_match(&["en", "fr", "it", "pt"], &[], None));

		// Both lists empty
		assert!(check_best_match(&[], &[], None));

		// More subtags
		assert!(check_best_match(&["zh", "zh-cmn", "zh-cmn-Hans"], &["zh-cmn-SG"], Some("zh-cmn")));
		assert!(check_best_match(&["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], &["zh-cmn-SG"], Some("zh-cmn-Hans-SG")));
		assert!(check_best_match(&["zh", "zh-cmn", "zh-cmn-Hans-SG"], &["zh-Hans"], Some("zh-cmn-Hans-SG")));
		assert!(check_best_match(&["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], &["zh-Hans"], Some("zh-cmn-Hans")));
		assert!(check_best_match(&["zh", "zh-cmn", "zh-cmn-Hans", "zh-cmn-Hans-SG"], &["zh-SG"], Some("zh-cmn-Hans-SG")));

		// Extensions
		assert!(check_best_match(&["zh", "he"], &["he-IL-u-ca-hebrew-tz-jeruslm", "zh"], Some("he")));
		assert!(check_best_match(&["zh", "he-IL-u-ca-hebrew-tz-jeruslm-nu-latn"], &["he", "zh"], Some("he-IL-u-ca-hebrew-tz-jeruslm-nu-latn")));
		assert!(check_best_match(&["ar-u-nu-latn", "ar"], &["ar-u-no-latn", "ar", "en-US", "en"], Some("ar-u-nu-latn")));
		assert!(check_best_match(&["fr-FR-u-em-text", "gsw-u-em-emoji"], &["gsw-u-em-text"], Some("gsw-u-em-emoji")));

		// Malformed
		assert!(check_best_match(&["en-US-SUS-BUS-VUS-GUS"], &["en"], None));
		assert!(check_best_match(&["en-abcdefghijklmnopqrstuvwxyz"], &["en"], None));
		assert!(check_best_match(&["ru-ЖЖЯЯ"], &["ru"], None));
		assert!(check_best_match(&["ru--"], &["ru"], None));
		assert!(check_best_match(&["", "@", "!!!", "721345"], &["en", "", "@", "!!!", "721345"], None));

		// Repeating
		assert!(check_best_match(&["en", "en", "en", "en"], &["ru-RU", "ru", "en-US", "en"], Some("en")));
		assert!(check_best_match(&["en-US", "en-GB", "ru-UA", "fr-FR", "it"], &["kk", "ru", "pt", "ru"], Some("ru-UA")));

		// Littered
		assert!(check_best_match(&["!!!!!!", "qwydgn12i6i", "ЖЖяяЖяЬЬЬ", "en-US", "!*&^^&*", "qweqweqweqwe-qweqwe", "ru-RU", "@@", "@"], &["ru", "en"], Some("ru-RU")));
		assert!(check_best_match(&["", "", "", "zh", "", "", "", "", "", "he", "", ""], &["he-IL-u-ca-hebrew-tz-jeruslm", "", "", "zh"], Some("he")));
		assert!(check_best_match(&["bla-!@#", "12345", "en-US", "en-GB", "ru-UA", "fr-FR", "it"], &["bla-!@#", "12345", "en-US", "en", "ru-RU", "ru"], Some("en-US")));
	}
}
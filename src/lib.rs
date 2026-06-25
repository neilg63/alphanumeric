use std::str::FromStr;

#[cfg(feature = "cell_analysis")]
pub mod cell_analysis;
mod char_type;
mod uses_decimal_comma;

#[cfg(feature = "cell_analysis")]
pub use cell_analysis::{analyze_cell, detect_column_format, CellAnalysis};
pub use char_type::CharType;
pub use to_segments::ToSegments;
pub use uses_decimal_comma::uses_decimal_comma;

/// Specifies how to interpret decimal separators when extracting numbers
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecimalSeparator {
    /// Treat `.` as the decimal separator (US, UK, Asia, etc.)
    Point,
    /// Treat `,` as the decimal separator (many European, Latin American, African countries)
    Comma,
    /// Intelligently detect based on separator patterns (recommended for mixed contexts)
    Auto,
}
/// Helper to add a sanitized numeric string (trims trailing separators)
fn add_sanitized_numeric_string(output: &mut Vec<String>, num_string: &str) {
    output.push(
        num_string
            .trim_end_matches(".")
            .trim_end_matches(",")
            .to_string(),
    );
}

/// Method to check if the string may be parsed to an integer or float
pub trait IsNumeric {
    /// strict check on a numeric string before using `.parse::<T>()`
    fn is_numeric(&self) -> bool;
}

impl<T: AsRef<str>> IsNumeric for T {
    fn is_numeric(&self) -> bool {
        let s = self.as_ref();
        let num_chars = s.chars().count();
        if num_chars < 1 {
            return false;
        }
        let last_index = num_chars - 1;
        let mut num_valid: usize = 0;
        let mut index: usize = 0;
        let mut num_decimal_separators = 0usize;
        for c in s.chars().into_iter() {
            let is_digit = c.is_digit(10);
            let valid_char = if is_digit {
                true
            } else {
                match c {
                    '-' => index == 0,
                    '.' => index < last_index && num_decimal_separators < 1,
                    _ => false,
                }
            };
            if c == '.' {
                num_decimal_separators += 1;
            }
            if valid_char {
                num_valid += 1;
            }
            index += 1;
        }
        num_valid == num_chars
    }
}

/// Set of methods to strip unwanted characters by type or extract vectors of numeric strings, integers or floats
pub trait StripCharacters<'a>
where
    Self: ToSegments,
{
    /// Remove all non-alphanumeric characters
    fn strip_non_alphanum(&self) -> String;
    /// Remove all non-digit characters
    fn strip_non_digits(&self) -> String;
    /// Remove whitespace characters
    fn strip_spaces(&self) -> String {
        self.strip_by_type(CharType::Spaces)
    }
    /// Remove characters matching a specific type
    fn strip_by_type(&self, ct: CharType<'a>) -> String;
    /// Remove characters matching any of the specified types
    fn strip_by_types(&self, cts: &[CharType<'a>]) -> String;
    /// Keep only characters matching a specific type
    fn filter_by_type(&self, ct: CharType<'a>) -> String;
    /// Keep only characters matching any of the specified types
    fn filter_by_types(&self, cts: &[CharType<'a>]) -> String;
    /// Extract numeric strings with configurable decimal separator
    fn to_numeric_strings_strict(&self, separator: DecimalSeparator) -> Vec<String>;
    /// Extract numeric strings using auto-detection
    fn to_numeric_strings(&self) -> Vec<String> {
        self.to_numeric_strings_strict(DecimalSeparator::Auto)
    }
    /// Parse extracted numbers with configurable decimal separator
    fn to_numbers_strict<N: FromStr>(&self, separator: DecimalSeparator) -> Vec<N>;
    /// Parse numbers using auto-detection
    fn to_numbers<N: FromStr>(&self) -> Vec<N> {
        self.to_numbers_strict::<N>(DecimalSeparator::Auto)
    }
    /// Split by pattern and extract the first number from each segment
    fn split_to_numbers<N: FromStr + Copy>(&self, pattern: &str) -> Vec<N>;
    /// Normalize numeric string separators based on decimal separator convention
    fn correct_numeric_string_strict(&self, separator: DecimalSeparator) -> String;
    /// Normalize numeric string separators using auto-detection
    fn correct_numeric_string(&self) -> String {
        self.correct_numeric_string_strict(DecimalSeparator::Auto)
    }
    /// Extract the first parsed number using auto-detection, or None if empty
    fn to_first_number<N: FromStr + Copy>(&self) -> Option<N> {
        if let Some(number) = self.to_numbers::<N>().first() {
            Some(*number)
        } else {
            None
        }
    }
    /// Extract the first parsed number with configurable decimal separator, or None if empty
    fn to_first_number_strict<N: FromStr + Copy>(&self, separator: DecimalSeparator) -> Option<N> {
        if let Some(number) = self.to_numbers_strict::<N>(separator).first() {
            Some(*number)
        } else {
            None
        }
    }
    /// Extract numeric strings and join them with spaces
    fn strip_non_numeric(&self) -> String {
        self.to_numeric_strings().join(" ")
    }
}

impl<'a, T: AsRef<str>> StripCharacters<'a> for T {
    fn strip_non_alphanum(&self) -> String {
        self.as_ref()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
    }

    fn strip_non_digits(&self) -> String {
        self.as_ref()
            .chars()
            .filter(|c| c.is_digit(10))
            .collect::<String>()
    }

    fn strip_by_type(&self, ct: CharType<'a>) -> String {
        self.as_ref()
            .chars()
            .filter(|c| ct.is_in_range(c) == false)
            .collect::<String>()
    }

    fn strip_by_types(&self, cts: &[CharType<'a>]) -> String {
        self.as_ref()
            .chars()
            .filter(|c| cts.iter().any(|ct| ct.is_in_range(c)) == false)
            .collect::<String>()
    }

    fn filter_by_type(&self, ct: CharType<'a>) -> String {
        self.as_ref()
            .chars()
            .filter(|c| ct.is_in_range(c))
            .collect::<String>()
    }

    fn filter_by_types(&self, cts: &[CharType<'a>]) -> String {
        self.as_ref()
            .chars()
            .filter(|c| cts.iter().any(|ct| ct.is_in_range(c)))
            .collect::<String>()
    }

    fn correct_numeric_string_strict(&self, separator: DecimalSeparator) -> String {
        let s = self.as_ref();
        let chars: Vec<char> = s.chars().collect();

        let mut separators: Vec<(usize, char)> = Vec::new();
        for (i, &ch) in chars.iter().enumerate() {
            if matches!(ch, '.' | ',' | ' ' | '\'' | '·' | '․') {
                let prev_is_digit = i > 0 && chars[i - 1].is_digit(10);
                let next_is_digit = i < chars.len() - 1 && chars[i + 1].is_digit(10);

                if prev_is_digit && next_is_digit {
                    separators.push((i, ch));
                }
            }
        }

        if separators.is_empty() {
            return s.to_string();
        }

        let decimal_idx = match separator {
            DecimalSeparator::Comma => separators
                .iter()
                .find(|(_, ch)| *ch == ',')
                .map(|(i, _)| *i),
            DecimalSeparator::Point => separators
                .iter()
                .find(|(_, ch)| *ch == '.')
                .map(|(i, _)| *i),
            DecimalSeparator::Auto => {
                if separators.len() == 1 {
                    Some(separators[0].0)
                } else {
                    let dot_count = separators.iter().filter(|(_, ch)| *ch == '.').count();
                    let comma_count = separators.iter().filter(|(_, ch)| *ch == ',').count();

                    if dot_count > 1 && comma_count == 1 {
                        separators
                            .iter()
                            .find(|(_, ch)| *ch == ',')
                            .map(|(i, _)| *i)
                    } else if comma_count > 1 && dot_count == 1 {
                        separators
                            .iter()
                            .find(|(_, ch)| *ch == '.')
                            .map(|(i, _)| *i)
                    } else {
                        Some(separators.last().unwrap().0)
                    }
                }
            }
        };

        let mut result = String::new();
        for (i, &ch) in chars.iter().enumerate() {
            if separators.iter().any(|(pos, _)| *pos == i) {
                if Some(i) == decimal_idx {
                    result.push('.');
                }
            } else {
                result.push(ch);
            }
        }

        result
    }

    fn to_numeric_strings_strict(&self, separator: DecimalSeparator) -> Vec<String> {
        let s = self.as_ref();
        let mut prev_char = ' ';
        let mut seq_num = 0;
        let mut num_string = String::new();
        let mut output: Vec<String> = Vec::new();
        let last_index = s.chars().count().checked_sub(1).unwrap_or(0);
        let mut index: usize = 0;
        let mut prev_is_separator = false;
        for component in s.chars() {
            let mut is_end = index == last_index;
            let is_digit = component.is_digit(10);
            if prev_is_separator && !is_digit {
                let num_str_len = num_string.len();
                if num_str_len > 1 {
                    num_string = (&num_string[0..num_str_len - 1]).to_string();
                    is_end = true;
                    seq_num = num_string.len();
                }
            }
            if is_digit {
                if prev_char == '-' {
                    num_string.push(prev_char);
                }
                num_string.push(component);
                seq_num += 1;
                prev_is_separator = false;
            } else if prev_char.is_digit(10) {
                match component {
                    '.' | '․' | ',' | ' ' | '\'' | '·' => {
                        if index == last_index {
                            is_end = true;
                        } else {
                            if component == ',' {
                                num_string.push(',');
                            } else {
                                num_string.push('.');
                            }
                            seq_num = 0;
                        }
                        prev_is_separator = true;
                    }
                    _ => {
                        is_end = true;
                    }
                }
            } else {
                is_end = true;
                prev_is_separator = false;
            }
            if is_end {
                if seq_num > 0 {
                    add_sanitized_numeric_string(
                        &mut output,
                        &num_string.correct_numeric_string_strict(separator),
                    );
                    num_string = String::new();
                    seq_num = 0;
                }
            }
            prev_char = component;
            index += 1;
        }
        output
    }

    fn to_numbers_strict<N: FromStr>(&self, separator: DecimalSeparator) -> Vec<N> {
        self.to_numeric_strings_strict(separator)
            .into_iter()
            .map(|s| s.parse::<N>())
            .filter_map(|s| s.ok())
            .collect()
    }

    fn split_to_numbers<N: FromStr + Copy>(&self, pattern: &str) -> Vec<N> {
        self.to_segments(pattern)
            .into_iter()
            .filter_map(|part| part.to_first_number::<N>())
            .collect::<Vec<N>>()
    }
}

/// Methods to validate strings with character classes
pub trait CharGroupMatch {
    fn has_digits(&self) -> bool;
    fn has_digits_radix(&self, radix: u8) -> bool;
    fn has_alphanumeric(&self) -> bool;
    fn has_alphabetic(&self) -> bool;
    fn is_digits_only(&self) -> bool;
    fn is_digits_only_radix(&self, radix: u8) -> bool;
}

impl<T: AsRef<str>> CharGroupMatch for T {
    fn has_digits(&self) -> bool {
        self.as_ref().chars().any(|c| c.is_ascii_digit())
    }

    fn has_digits_radix(&self, radix: u8) -> bool {
        self.as_ref().chars().any(|c| c.is_digit(radix as u32))
    }

    fn has_alphanumeric(&self) -> bool {
        self.as_ref().chars().any(char::is_alphanumeric)
    }

    fn has_alphabetic(&self) -> bool {
        self.as_ref().chars().any(char::is_alphabetic)
    }

    fn is_digits_only(&self) -> bool {
        self.as_ref().chars().all(|c| c.is_ascii_digit())
    }

    fn is_digits_only_radix(&self, radix: u8) -> bool {
        self.as_ref().chars().all(|c| c.is_digit(radix as u32))
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // IsNumeric tests
    #[test]
    fn test_is_numeric() {
        assert!("123".is_numeric());
        assert!("-456".is_numeric());
        assert!("12.34".is_numeric());
        assert!(!"12.34.56".is_numeric());
        assert!(!"12a34".is_numeric());
    }

    #[test]
    fn test_is_numeric_empty() {
        assert!(!"".is_numeric());
    }

    #[test]
    fn test_is_numeric_comprehensive() {
        let num_str_1 = "-1227.75";
        assert!(num_str_1.is_numeric());

        let num_str_2 = "-1,227.75";
        assert_eq!(num_str_2.is_numeric(), false);
        assert!(num_str_2
            .correct_numeric_string_strict(DecimalSeparator::Point)
            .is_numeric());

        let num_str_3 = "-1.227,75";
        assert!(num_str_3
            .correct_numeric_string_strict(DecimalSeparator::Comma)
            .is_numeric());

        let num_str_4 = "$19.99 each";
        assert!(!num_str_4.is_numeric());
    }

    // StripCharacters tests
    #[test]
    fn test_strip_non_alphanum() {
        let text = "hello123!@#world";
        assert_eq!(text.strip_non_alphanum(), "hello123world");
    }

    #[test]
    fn test_strip_non_alphanum_unicode() {
        let source_str = "Cañon, Zürich, Москва";
        let target_str = "CañonZürichМосква";
        assert_eq!(source_str.strip_non_alphanum(), target_str);
    }

    #[test]
    fn test_strip_non_digits() {
        let text = "hello123!@#456";
        assert_eq!(text.strip_non_digits(), "123456");
    }

    // International format tests
    #[test]
    fn test_french_space_comma() {
        let french = "Le prix est 19 999,99 euros";
        let nums: Vec<f64> = french.to_numbers();
        assert_eq!(nums, vec![19999.99], "Failed: French space+comma format");
    }

    #[test]
    fn test_swiss_apostrophe_dot() {
        let swiss = "The cost is CHF 19'999.99";
        let nums: Vec<f64> = swiss.to_numbers();
        assert_eq!(nums, vec![19999.99], "Failed: Swiss apostrophe format");
    }

    #[test]
    fn test_european_dot_comma() {
        let european = "Kaufpreis: 1.500,50 EUR";
        let nums: Vec<f64> = european.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums, vec![1500.50], "Failed: European format");
    }

    #[test]
    fn test_middle_dot_separator() {
        let middle_dot = "Preu: 1·234,56";
        let nums: Vec<f64> = middle_dot.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums, vec![1234.56], "Failed: Middle dot format");
    }

    #[test]
    fn test_apostrophe_in_text_preserved() {
        let text = "È l'ingresso costa 15,00 euro";
        let corrected = text.correct_numeric_string_strict(DecimalSeparator::Comma);
        assert_eq!(
            corrected, "È l'ingresso costa 15.00 euro",
            "Failed: Apostrophe in text should be preserved"
        );
    }

    #[test]
    fn test_mixed_formats() {
        let mixed = "Prices: 75.02029 or 12,345.67 or 1.234,56 EUR";
        let nums: Vec<f64> = mixed.to_numbers();
        assert!(nums.len() >= 1, "Should extract at least one number");
    }

    #[test]
    fn test_numeric_extraction_with_space_separator() {
        let text = "Values: 1 234 567,89 or 100 000";
        let nums: Vec<String> = text.to_numeric_strings_strict(DecimalSeparator::Comma);
        assert!(
            nums.len() >= 1,
            "Should extract numeric strings with spaces"
        );
        let corrected: Vec<String> = nums
            .iter()
            .map(|s| s.correct_numeric_string_strict(DecimalSeparator::Comma))
            .collect();
        assert!(
            !corrected.is_empty(),
            "Should produce corrected numeric strings"
        );
    }

    #[test]
    fn test_unambiguous_european_multiple_dots_single_comma() {
        let num_str = "1.999.999,25";
        let corrected = num_str.correct_numeric_string();
        assert_eq!(
            corrected, "1999999.25",
            "Multiple dots + single comma should be decimal-comma"
        );
    }

    #[test]
    fn test_unambiguous_english_multiple_commas_single_dot() {
        let num_str = "1,999,999.25";
        let corrected = num_str.correct_numeric_string();
        assert_eq!(
            corrected, "1999999.25",
            "Multiple commas + single dot should be decimal-point"
        );
    }

    #[test]
    fn test_ambiguous_pattern_fallback_to_last() {
        let num_str = "19,99,999.25";
        let corrected = num_str.correct_numeric_string();
        assert_eq!(
            corrected, "1999999.25",
            "Should treat last separator as decimal"
        );
    }

    #[test]
    fn test_single_ambiguous_separator() {
        let num_str = "1,356";
        let corrected = num_str.correct_numeric_string();
        assert_eq!(
            corrected, "1.356",
            "Single separator should be treated as decimal"
        );
    }

    #[test]
    fn test_space_separated_with_comma_decimal() {
        let num_str = "345 789,98";
        let corrected = num_str.correct_numeric_string();
        assert_eq!(corrected, "345789.98", "Space + comma pattern should work");
    }

    #[test]
    fn test_extract_unambiguous_numbers() {
        let text = "Europe prices: 1.999.999,25 and 1.500,50 USD";
        let nums: Vec<f64> = text.to_numbers();
        assert!(
            nums.iter().any(|&n| (n - 1999999.25).abs() < 0.001),
            "Should extract 1.999.999,25"
        );
        assert!(
            nums.iter().any(|&n| (n - 1500.50).abs() < 0.001),
            "Should extract 1.500,50"
        );
    }

    #[test]
    fn test_correct_floats() {
        let source_str = "Ho pagato 15,00€ per l'ingresso.";
        let target_str = "Ho pagato 15.00€ per l'ingresso.";
        assert_eq!(
            source_str.correct_numeric_string_strict(DecimalSeparator::Comma),
            target_str
        );

        let source_str_2 = "Pesa 1.678 grammi";
        let target_str_2 = "Pesa 1678 grammi";
        assert_eq!(
            source_str_2.correct_numeric_string_strict(DecimalSeparator::Comma),
            target_str_2
        );

        let sample_str = "Ho pagato 12,50€ per 1.500 grammi di sale.";
        let target_numbers = vec![12.5f32, 1500f32];
        assert_eq!(
            sample_str.to_numbers_strict::<f32>(DecimalSeparator::Comma),
            target_numbers
        );
    }

    #[test]
    fn test_split_to_numbers() {
        let source_str = "75.02029,-9.2928";
        let numbers = source_str.split_to_numbers::<f64>(",");
        let expected_vec = vec![75.02029, -9.2928];
        assert_eq!(numbers, expected_vec);

        let source_str_2 = "bag 74.99, orange 9.29 bank balance -1.229,89";
        let expected_vec_2 = vec![74.99, 9.29, -1229.89];
        assert_eq!(source_str_2.to_numbers::<f64>(), expected_vec_2);
    }

    #[test]
    fn test_strip_non_numeric_comprehensive() {
        let source_str = "I spent £9999.99 on 2 motorbikes at the age of 72.";
        let target_str = "9999.99 2 72";
        assert_eq!(source_str.strip_non_numeric(), target_str);

        assert_eq!(
            source_str.to_numbers::<f64>(),
            vec![9999.99f64, 2f64, 72f64]
        );

        assert_eq!(
            source_str.to_first_number::<f32>().unwrap_or(0f32),
            9999.99f32
        );

        let input_text = "I'd like 2.5lb of flour please";
        assert_eq!(input_text.to_first_number::<f32>().unwrap_or(0f32), 2.5f32);

        let input_text = "Il conto è del 1.999,50€. Come vuole pagare?";
        assert_eq!(
            input_text.to_first_number::<f32>().unwrap_or(0f32),
            1999.5f32
        );

        let input_text = "Il furgone pesa 1.500kg, ma costa solo 19.900€";
        assert_eq!(
            input_text
                .to_first_number_strict::<u32>(DecimalSeparator::Comma)
                .unwrap_or(0),
            1500
        );

        assert_eq!(
            input_text.to_numbers_strict::<u32>(DecimalSeparator::Comma),
            vec![1_500, 19_900]
        );
    }

    #[test]
    fn test_char_group_matches() {
        assert!("abc123".has_digits());
        assert!(!"abc".has_digits());
        assert!("abc123".has_alphanumeric());
        assert!("abc".has_alphabetic());
        assert!("123".is_digits_only());
        assert!(!"12a3".is_digits_only());
    }

    #[test]
    fn test_has_digits_comprehensive() {
        let num_str_1 = "serial number: 93025371";
        assert!(num_str_1.has_digits());

        let num_str_2 = "93025371";
        assert!(num_str_2.is_digits_only());

        let num_str_3 = "1ec9F9a";
        assert!(num_str_3.is_digits_only_radix(16));
    }

    // Decimal separator variant tests
    #[test]
    fn test_point_as_decimal_us_convention() {
        let text = "Price: $1,234.56 and 999.99 items";
        let nums: Vec<f64> = text.to_numbers();
        assert_eq!(nums.len(), 2);
        assert!(nums.iter().any(|&n| (n - 1234.56).abs() < 0.001));
        assert!(nums.iter().any(|&n| (n - 999.99).abs() < 0.001));
    }

    #[test]
    fn test_comma_as_decimal_european_convention() {
        let text = "Preço: R$ 1.234,56 e 999,99 itens";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums.len(), 2);
        assert!(nums.iter().any(|&n| (n - 1234.56).abs() < 0.001));
        assert!(nums.iter().any(|&n| (n - 999.99).abs() < 0.001));
    }

    #[test]
    fn test_space_thousands_point_decimal() {
        let text = "Amount: 1 234.56 USD";
        let nums: Vec<f64> = text.to_numbers();
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_space_thousands_comma_decimal() {
        let text = "Montant: 1 234,56 EUR";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_both_variants_in_mixed_context() {
        // Point-as-decimal context
        let us_text = "Values: 1,000.50 and 2,999.75";
        let us_nums: Vec<f64> = us_text.to_numbers();
        assert_eq!(us_nums.len(), 2);
        assert!(us_nums.iter().any(|&n| (n - 1000.50).abs() < 0.001));

        // Comma-as-decimal context
        let eu_text = "Valores: 1.000,50 e 2.999,75";
        let eu_nums: Vec<f64> = eu_text.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(eu_nums.len(), 2);
        assert!(eu_nums.iter().any(|&n| (n - 1000.50).abs() < 0.001));
    }

    // DecimalSeparator enum tests
    #[test]
    fn test_decimal_separator_auto_detection() {
        // Mixed separators -> auto-detects decimal as comma
        let text = "Amount: 1.234,56";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Auto);
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_decimal_separator_explicit_point() {
        // Force point as decimal
        let text = "Value: 1,234.56";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Point);
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_decimal_separator_explicit_comma() {
        // Force comma as decimal
        let text = "Value: 1.234,56";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_decimal_separator_point_only() {
        // When point is forced, treat other separators as thousands
        let text = "1.234,56 dollars";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Point);
        // Should parse as 1.234, stopping at comma
        assert_eq!(nums.len(), 1);
        assert!((nums[0] - 1.234).abs() < 0.001 || (nums[0] - 1234.0).abs() < 0.001);
    }

    #[test]
    fn test_decimal_separator_comma_only() {
        // When comma is forced, treat other separators as thousands
        let text = "1.234,56 euros";
        let nums: Vec<f64> = text.to_numbers_strict(DecimalSeparator::Comma);
        assert_eq!(nums, vec![1234.56]);
    }

    #[test]
    fn test_numeric_strings_strict_auto() {
        let text = "Prices: 1.234,56 and 999,99";
        let nums: Vec<String> = text.to_numeric_strings_strict(DecimalSeparator::Auto);
        assert_eq!(nums.len(), 2);
        assert!(nums.iter().any(|s| s == "1234.56" || s == "1234,56"));
    }

    #[test]
    fn test_first_number_strict() {
        let text = "First: 123.45, Second: 678.90";
        let first: Option<f64> = text.to_first_number_strict(DecimalSeparator::Point);
        assert_eq!(first, Some(123.45));
    }

    #[test]
    fn test_correct_numeric_string_strict_auto() {
        let num_str = "1.999,99";
        let corrected = num_str.correct_numeric_string_strict(DecimalSeparator::Auto);
        assert_eq!(corrected, "1999.99");
    }

    #[test]
    fn test_correct_numeric_string_strict_point() {
        let num_str = "1.999,99";
        let corrected = num_str.correct_numeric_string_strict(DecimalSeparator::Point);
        // With point as decimal, should keep only the point
        assert!(corrected.contains('.'));
    }

    #[test]
    fn test_correct_numeric_string_strict_comma() {
        let num_str = "1.999,99";
        let corrected = num_str.correct_numeric_string_strict(DecimalSeparator::Comma);
        assert_eq!(corrected, "1999.99");
    }
}

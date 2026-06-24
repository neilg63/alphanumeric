[![mirror](https://img.shields.io/badge/mirror-github-blue)](https://github.com/neilg63/alphanumeric)
[![crates.io](https://img.shields.io/crates/v/alphanumeric.svg)](https://crates.io/crates/alphanumeric)
[![docs.rs](https://docs.rs/alphanumeric/badge.svg)](https://docs.rs/alphanumeric)

# Alphanumeric

This library provides character filtering, numeric validation, and international number extraction for Rust strings. It builds on the Rust standard library with no regex dependency. It focuses on stripping or filtering characters by type and parsing numbers from free text, with full support for international decimal and thousands separator conventions (points, commas, spaces, apostrophes, middle dots).

## Dependencies

- [**to-segments**](https://crates.io/crates/to_segments) — Splits string slices and owned strings into vectors and extracts optional strings, pairs of optional strings.

## Related crates

- [**to-segments**](https://crates.io/crates/to_segments) — Provides the `ToSegments` trait for ergonomic string splitting with readable methods for common manipulation tasks. Also used internally by `alphanumeric`.
- [**enclose-strings**](https://crates.io/crates/enclose-strings) — Wrap or enclose strings in matching or complementary characters with optional escaping.
- [**simple-string-patterns**](https://crates.io/crates/simple-string-patterns) — Provides simple methods to match and filter strings by simple patterns without regular expressions.

## Method overview

| Component<br /><sup>position</sup> | Meaning                                                   |
| ---------------------------------- | --------------------------------------------------------- |
| strip_by\_ <sub>⇤</sub>            | Return a string without the specified character type(s)   |
| filter_by\_ <sub>⇤</sub>           | Return a string with only the specified character type(s) |
| to_numbers <sub>⇤</sub>            | Extract and parse numeric values from free text           |
| to_numeric_strings <sub>⇤</sub>    | Extract numeric substrings without parsing                |
| \_strict <sub>⇥</sub>              | Variant accepting an explicit `DecimalSeparator` argument |

## Examples

### Check if a string is a valid number

```rust
use alphanumeric::IsNumeric;

let num_str = "-1227.75";
assert!(num_str.is_numeric()); // true

let num_str = "12a34";
assert!(!num_str.is_numeric()); // false — contains non-numeric character
```

### Extract the first decimal value from a longer string

```rust
use alphanumeric::StripCharacters;

const GBP_TO_EURO: f64 = 0.835;
let sample_str = "Price £12.50 each";
if let Some(price_gbp) = sample_str.to_first_number::<f64>() {
    let price_eur = price_gbp / GBP_TO_EURO;
    println!("The price in euros is {:.2}", price_eur);
}
```

### Extract numeric sequences from phrases and convert them to a vector of floats

```rust
use alphanumeric::{StripCharacters, DecimalSeparator};

// Extract European-style numbers with commas as decimal separators
// and dots as thousand separators
let sample_str = "2.500 grammi di farina costa 9,90€ al supermercato.";
let numbers: Vec<f32> = sample_str.to_numbers_strict(DecimalSeparator::Comma);
if numbers.len() > 1 {
    let weight_grams = numbers[0];
    let price_euros = numbers[1];
    let price_per_kg = price_euros / (weight_grams / 1000f32);
    println!("Flour costs €{:.2} per kilo", price_per_kg); // €3.96
}
```

### Split a string list of numbers into floats

```rust
use alphanumeric::StripCharacters;

// Extract 64-bit floats from a comma-separated list.
// Numbers within each segment are evaluated separately.
let sample_str = "34.2929,-93.701";
let numbers = sample_str.split_to_numbers::<f64>(",");
// yields vec![34.2929, -93.701]
```

### Extract all numbers from a sentence

```rust
use alphanumeric::StripCharacters;

let source_str = "I spent £9999.99 on 2 motorbikes at the age of 72.";
assert_eq!(source_str.strip_non_numeric(), "9999.99 2 72");
assert_eq!(source_str.to_numbers::<f64>(), vec![9999.99, 2.0, 72.0]);
assert_eq!(source_str.to_first_number::<f32>().unwrap(), 9999.99f32);
```

### International number formats

The `DecimalSeparator` enum controls how ambiguous separators are interpreted:

```rust
use alphanumeric::{StripCharacters, DecimalSeparator};

// US/UK format: commas as thousands, dot as decimal
let us_text = "Price: $1,234.56";
let nums: Vec<f64> = us_text.to_numbers();
assert_eq!(nums, vec![1234.56]);

// European format: dots as thousands, comma as decimal
let eu_text = "Kaufpreis: 1.500,50 EUR";
let nums: Vec<f64> = eu_text.to_numbers_strict(DecimalSeparator::Comma);
assert_eq!(nums, vec![1500.50]);

// French format: spaces as thousands, comma as decimal
let fr_text = "Le prix est 19 999,99 euros";
let nums: Vec<f64> = fr_text.to_numbers();
assert_eq!(nums, vec![19999.99]);

// Swiss format: apostrophes as thousands
let ch_text = "The cost is CHF 19'999.99";
let nums: Vec<f64> = ch_text.to_numbers();
assert_eq!(nums, vec![19999.99]);
```

### Normalize numeric strings

```rust
use alphanumeric::StripCharacters;

// Correct international separators to standard dot-decimal format
let european = "1.999.999,25";
assert_eq!(european.correct_numeric_string(), "1999999.25");

let english = "1,999,999.25";
assert_eq!(english.correct_numeric_string(), "1999999.25");
```

### Filter strings by character categories

```rust
use alphanumeric::{StripCharacters, CharType};

let sample_str = "Products: $9.99 per unit, £19.50 each, €15 only. Zürich café cañon";

let vowels_only = sample_str.filter_by_type(
    CharType::Chars(&['a', 'e', 'i', 'o', 'u', 'é', 'ü', 'y'])
);
// yields "oueuieaoyüiaéao"

let lower_a_to_m = sample_str.filter_by_type(CharType::Range('a'..'n'));
// yields "dceieachlichcafca"

// Filter by multiple character categories
let lower_and_spaces = sample_str.filter_by_types(
    &[CharType::Lower, CharType::Spaces]
);
// yields "roducts  per unit  each  only ürich café cañon"
```

### Strip spaces

```rust
use alphanumeric::StripCharacters;

let sample_str = "19 May 2021 ";
let without_spaces = sample_str.strip_spaces();
// yields "19May2021"
```

### Remove character categories from strings

```rust
use alphanumeric::{StripCharacters, CharType};

let sample_str = "Products: $9.99 per unit, £19.50 each, €15 only. Zürich café cañon";

let without_punctuation = sample_str.strip_by_type(CharType::Punctuation);
// yields "Products 999 per unit £1950 each €15 only Zürich café cañon"

let without_spaces_and_punct = sample_str.strip_by_types(
    &[CharType::Spaces, CharType::Punctuation]
);
// yields "Products999perunit£1950each€15onlyZürichcafécañon"
```

### Validate strings with character classes

```rust
use alphanumeric::CharGroupMatch;

assert!("abc123".has_digits());       // true — contains digit characters
assert!(!"abc".has_digits());          // false

assert!("abc123".has_alphanumeric()); // true
assert!("abc".has_alphabetic());      // true

assert!("123".is_digits_only());      // true
assert!(!"12a3".is_digits_only());    // false

// Hexadecimal digit validation
assert!("1ec9F9a".is_digits_only_radix(16)); // true
```

### Detect decimal separator format across a column of data

```rust
use alphanumeric::{analyze_cell, detect_column_format, CellAnalysis, DecimalSeparator};

// Analyze individual cells
assert_eq!(analyze_cell("1 234,56"), CellAnalysis::Comma);
assert_eq!(analyze_cell("1'234.56"), CellAnalysis::Point);
assert_eq!(analyze_cell("1.234,56"), CellAnalysis::Comma);

// Detect format across a column of values
let cells = vec!["1 234,56", "2 345,78", "3 456,90"];
assert_eq!(detect_column_format(&cells, 10), DecimalSeparator::Comma);
```

## Traits

| Name            | No. of methods | Description                                                                                                   |
| --------------- | -------------- | ------------------------------------------------------------------------------------------------------------- |
| IsNumeric       | 1              | Check if a string can be parsed to an integer or float                                                        |
| StripCharacters | 17             | Strip unwanted characters by type, or extract vectors of numeric strings, integers or floats                  |
| CharGroupMatch  | 6              | Validate strings with character classes: `has_digits`, `has_alphanumeric`, `has_alphabetic`, `is_digits_only` |

### IsNumeric

Strict check on a numeric string before using `.parse::<T>()`. Returns `true` for integers, negative numbers, and decimals with a single `.` separator.

```rust
use alphanumeric::IsNumeric;

assert!("123".is_numeric());
assert!("-456".is_numeric());
assert!("12.34".is_numeric());
assert!(!"12.34.56".is_numeric()); // multiple decimal points
assert!(!"".is_numeric());         // empty string
```

### StripCharacters

Strip unwanted characters by type or extract vectors of numeric strings, integers or floats without regular expressions.

| Method                               | Description                                                     |
| ------------------------------------ | --------------------------------------------------------------- |
| `strip_non_alphanum()`               | Remove all non-alphanumeric characters                          |
| `strip_non_digits()`                 | Remove all non-digit characters                                 |
| `strip_spaces()`                     | Remove whitespace characters                                    |
| `strip_by_type(ct)`                  | Remove characters matching a specific `CharType`                |
| `strip_by_types(cts)`                | Remove characters matching any of the specified `CharType`s     |
| `filter_by_type(ct)`                 | Keep only characters matching a specific `CharType`             |
| `filter_by_types(cts)`               | Keep only characters matching any of the specified `CharType`s  |
| `to_numeric_strings()`               | Extract numeric substrings using auto-detection                 |
| `to_numeric_strings_strict(sep)`     | Extract numeric substrings with explicit `DecimalSeparator`     |
| `to_numbers::<T>()`                  | Parse extracted numbers using auto-detection                    |
| `to_numbers_strict::<T>(sep)`        | Parse extracted numbers with explicit `DecimalSeparator`        |
| `to_first_number::<T>()`             | Extract the first parsed number, or `None`                      |
| `to_first_number_strict::<T>(sep)`   | Extract the first parsed number with explicit separator         |
| `split_to_numbers::<T>(pattern)`     | Split by pattern and extract the first number from each segment |
| `strip_non_numeric()`                | Extract numeric strings and join them with spaces               |
| `correct_numeric_string()`           | Normalize separators to standard dot-decimal format             |
| `correct_numeric_string_strict(sep)` | Normalize separators with explicit `DecimalSeparator`           |

### CharGroupMatch

Validate strings with character classes.

| Method                        | Description                                    |
| ----------------------------- | ---------------------------------------------- |
| `has_digits()`                | Contains at least one ASCII digit              |
| `has_digits_radix(radix)`     | Contains at least one digit in the given radix |
| `has_alphanumeric()`          | Contains at least one alphanumeric character   |
| `has_alphabetic()`            | Contains at least one alphabetic character     |
| `is_digits_only()`            | All characters are ASCII digits                |
| `is_digits_only_radix(radix)` | All characters are digits in the given radix   |

## Enums

### DecimalSeparator

Controls how ambiguous separators (dots, commas) are interpreted when extracting numbers.

| Variant | Meaning                                                                               |
| ------- | ------------------------------------------------------------------------------------- |
| `Point` | Treat `.` as the decimal separator (US, UK, Asia, etc.)                               |
| `Comma` | Treat `,` as the decimal separator (many European, Latin American, African countries) |
| `Auto`  | Intelligently detect based on separator patterns (recommended for mixed contexts)     |

### CharType

Defines categories, sets or ranges of characters as well as single characters. Used with `strip_by_type`, `strip_by_types`, `filter_by_type`, and `filter_by_types`.

| Variant       | Arguments       | Meaning                                                                 |
| ------------- | --------------- | ----------------------------------------------------------------------- |
| `Any`         | —               | Matches any character                                                   |
| `DecDigit`    | —               | Match 0–9 only (`is_ascii_digit`)                                       |
| `Digit`       | `(u32)`         | Match digit with the specified radix (e.g. 16 for hexadecimal)          |
| `Numeric`     | —               | Match number-like characters in the decimal base (excludes `.` and `-`) |
| `AlphaNum`    | —               | Match any alphanumeric characters (`is_alphanumeric`)                   |
| `Lower`       | —               | Match lower case letters (`is_lowercase`)                               |
| `Upper`       | —               | Match upper case letters (`is_uppercase`)                               |
| `Alpha`       | —               | Match any letters in most supported alphabets (`is_alphabetic`)         |
| `Spaces`      | —               | Match whitespace (`is_whitespace`)                                      |
| `Punctuation` | —               | Match ASCII punctuation (`is_ascii_punctuation`)                        |
| `Char`        | `(char)`        | Match a single character                                                |
| `Chars`       | `(&[char])`     | Match an array of characters                                            |
| `Range`       | `(Range<char>)` | Match a range, e.g. `'a'..'d'` includes a, b, c but not d               |
| `Between`     | `(char, char)`  | Match characters between the specified bounds, inclusive on both ends   |

### CellAnalysis

Result of analyzing a single cell's decimal separator format, returned by `analyze_cell()`.

| Variant  | Meaning                               |
| -------- | ------------------------------------- |
| `Point`  | Clearly uses `.` as decimal separator |
| `Comma`  | Clearly uses `,` as decimal separator |
| `Either` | Ambiguous — could use either format   |
| `None`   | No decimal separator present          |

## Functions

### analyze_cell

`analyze_cell(txt: &str) -> CellAnalysis`

Analyze a single numeric string to determine which decimal separator convention it uses. Examines the positions of dots, commas, spaces, apostrophes and middle dots to make a determination.

### detect_column_format

`detect_column_format(cells: &[&str], max_scan: usize) -> DecimalSeparator`

Scan a column of cell values and detect the predominant decimal separator format. Stops early when the format becomes clear (3+ votes with a 2+ lead). Useful for batch processing tabular data where the format is consistent within a column.

### uses_decimal_comma

`uses_decimal_comma(txt: &str, enforce_euro_mode: bool) -> bool`

Detect if a numeric string uses European format with `,` as the decimal separator and dots as thousand separators. When `enforce_euro_mode` is `true`, a single comma is always treated as a decimal separator; otherwise it may be interpreted as a thousands separator depending on position.

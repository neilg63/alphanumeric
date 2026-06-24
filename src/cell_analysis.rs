use crate::DecimalSeparator;

/// Analysis result for a single cell—what format evidence it shows
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellAnalysis {
    /// Clearly uses `.` as decimal separator
    Point,
    /// Clearly uses `,` as decimal separator
    Comma,
    /// Ambiguous—could use either format
    Either,
    /// No decimal separator present
    None,
}

/// Analyze a single cell to determine its decimal separator usage
pub fn analyze_cell(txt: &str) -> CellAnalysis {
    let has_space = txt.contains(' ');
    let has_apostrophe = txt.contains('\'');
    let has_middle_dot = txt.contains('·');

    let num_dots = txt.matches('.').count();
    let num_commas = txt.matches(',').count();

    match (
        num_dots,
        num_commas,
        has_space,
        has_apostrophe,
        has_middle_dot,
    ) {
        // Clear cases: other thousands separator + one decimal candidate
        (0, 1, true, _, _) => CellAnalysis::Comma, // "1 234,56"
        (1, 0, _, true, _) => CellAnalysis::Point, // "1'234.56"
        (1, 0, _, _, true) => CellAnalysis::Point, // "1234·56" (middle dot is thousands)

        // No separators
        (0, 0, _, _, _) => CellAnalysis::None,

        // Multiple dots or commas (thousands separators)
        (d, 0, _, _, _) if d > 1 => CellAnalysis::Point, // "1.234.567"
        (0, c, _, _, _) if c > 1 => CellAnalysis::Comma, // "1,234,567"

        // Both present: position determines which is decimal
        (d, c, _, _, _) if d > 0 && c > 0 => {
            if let (Some(dot_pos), Some(comma_pos)) = (txt.find('.'), txt.find(',')) {
                if dot_pos < comma_pos {
                    CellAnalysis::Comma // "1.234,56"
                } else {
                    CellAnalysis::Point // "1,234.56"
                }
            } else {
                CellAnalysis::Either
            }
        }

        // Single separator: check digits after it
        (1, 0, _, _, _) => digits_after_separator(txt, '.'),
        (0, 1, _, _, _) => digits_after_separator(txt, ','),

        _ => CellAnalysis::Either,
    }
}

/// Check digits after a separator to determine if it's a decimal or thousands separator
fn digits_after_separator(txt: &str, sep: char) -> CellAnalysis {
    if let Some(pos) = txt.rfind(sep) {
        let after = txt.len() - pos - 1;
        match after {
            0 => CellAnalysis::Either, // "1234," - ends with separator
            1 | 2 => {
                // "1234,5" or "1234,56" - likely decimal
                if sep == ',' {
                    CellAnalysis::Comma
                } else {
                    CellAnalysis::Point
                }
            }
            3 => CellAnalysis::Either, // "1234,567" - ambiguous (could be thousands)
            4.. => {
                // "1234,5678" - too many for thousands
                if sep == ',' {
                    CellAnalysis::Comma
                } else {
                    CellAnalysis::Point
                }
            }
        }
    } else {
        CellAnalysis::Either
    }
}

/// Scan column cells and detect the decimal separator format
/// Stops early when the format becomes clear (threshold exceeded)
pub fn detect_column_format(cells: &[&str], max_scan: usize) -> DecimalSeparator {
    let mut point_votes = 0;
    let mut comma_votes = 0;

    for cell in cells.iter().take(max_scan) {
        match analyze_cell(cell) {
            CellAnalysis::Point => {
                point_votes += 1;
                // Stop early if clear majority
                if point_votes > comma_votes + 2 && point_votes >= 3 {
                    return DecimalSeparator::Point;
                }
            }
            CellAnalysis::Comma => {
                comma_votes += 1;
                // Stop early if clear majority
                if comma_votes > point_votes + 2 && comma_votes >= 3 {
                    return DecimalSeparator::Comma;
                }
            }
            CellAnalysis::Either | CellAnalysis::None => {} // Skip ambiguous/empty
        }
    }

    // Fallback after scanning all cells
    match (comma_votes, point_votes) {
        (c, p) if c > p => DecimalSeparator::Comma,
        (c, p) if p > c => DecimalSeparator::Point,
        _ => DecimalSeparator::Auto,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_cell_clear_comma() {
        assert_eq!(analyze_cell("1 234,56"), CellAnalysis::Comma); // space + comma
        assert_eq!(analyze_cell("1234,56"), CellAnalysis::Comma); // 2 digits after
    }

    #[test]
    fn test_analyze_cell_clear_point() {
        assert_eq!(analyze_cell("1'234.56"), CellAnalysis::Point); // apostrophe + point
        assert_eq!(analyze_cell("1234.56"), CellAnalysis::Point); // 2 digits after
    }

    #[test]
    fn test_analyze_cell_ambiguous() {
        assert_eq!(analyze_cell("123,456"), CellAnalysis::Either); // 3 digits after (thousands?)
        assert_eq!(analyze_cell("1,234"), CellAnalysis::Either); // 3 digits after
    }

    #[test]
    fn test_analyze_cell_multiple_separators() {
        assert_eq!(analyze_cell("1.234.567"), CellAnalysis::Point); // multiple dots
        assert_eq!(analyze_cell("1,234,567"), CellAnalysis::Comma); // multiple commas
        assert_eq!(analyze_cell("1.234,56"), CellAnalysis::Comma); // dot first, comma is decimal
        assert_eq!(analyze_cell("1,234.56"), CellAnalysis::Point); // comma first, dot is decimal
    }

    #[test]
    fn test_detect_column_format_clear_comma() {
        let cells = vec!["1 234,56", "2 345,78", "3 456,90"];
        assert_eq!(detect_column_format(&cells, 10), DecimalSeparator::Comma);
    }

    #[test]
    fn test_detect_column_format_clear_point() {
        let cells = vec!["1'234.56", "2'345.78", "3'456.90"];
        assert_eq!(detect_column_format(&cells, 10), DecimalSeparator::Point);
    }

    #[test]
    fn test_detect_column_format_with_ambiguous() {
        // Mix of clear and ambiguous—clear ones win
        let cells = vec!["1 234,56", "123,456", "2 345,78"];
        assert_eq!(detect_column_format(&cells, 10), DecimalSeparator::Comma);
    }

    #[test]
    fn test_detect_column_format_stops_early() {
        // Should return early once 3+ votes with 2+ lead
        let cells = vec!["1 234,56", "2 345,78", "999"]; // Only needs first 2
        assert_eq!(detect_column_format(&cells, 10), DecimalSeparator::Comma);
    }
}

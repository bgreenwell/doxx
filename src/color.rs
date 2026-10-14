//! Shared color-parsing helpers.
//!
//! `parse_hex_rgb` was previously duplicated between `widgets::document`
//! (ratatui `Color::Rgb`) and `ansi` (crossterm `Color`, depth-aware) - both
//! need the same "#RRGGBB" -> (u8, u8, u8) parsing, they just build different
//! output types from the result.

/// Parse a "#RRGGBB" or "RRGGBB" hex color string into (r, g, b) bytes.
/// Returns `None` for anything that isn't exactly 6 hex digits.
pub(crate) fn parse_hex_rgb(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return None;
    }

    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

    Some((r, g, b))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_with_and_without_hash_prefix() {
        assert_eq!(parse_hex_rgb("#FF0000"), Some((255, 0, 0)));
        assert_eq!(parse_hex_rgb("FF0000"), Some((255, 0, 0)));
    }

    #[test]
    fn parses_mixed_case() {
        assert_eq!(parse_hex_rgb("#00ff80"), Some((0, 255, 128)));
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(parse_hex_rgb("#FFF"), None);
        assert_eq!(parse_hex_rgb("#FF00000"), None);
        assert_eq!(parse_hex_rgb(""), None);
    }

    #[test]
    fn rejects_non_hex_characters() {
        assert_eq!(parse_hex_rgb("#GGGGGG"), None);
    }
}

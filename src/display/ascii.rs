/// 22x7 ASCII art renderer for NFC tag data.
///
/// Renders tag information into a fixed-width 22-column by 7-row grid
/// using ASCII box-drawing characters (+, -, |). The 22-column width
/// fits small e-ink displays (e.g. Waveshare 2.13" at 250x122 px).
///
/// # Example output
/// ```text
/// +--------------------+
/// | NFC TAG DETECTED   |
/// |                    |
/// | UID 7ed59290       |
/// | TYPE Mifare Classic|
/// | tap #1             |
/// +--------------------+
/// ```

use crate::nfc::TagInfo;

/// Width of the display grid in characters.
pub const DISPLAY_WIDTH: usize = 22;

/// Height of the display grid in characters.
pub const DISPLAY_HEIGHT: usize = 7;

/// Maximum length for the tag type string before truncation.
const MAX_TAG_TYPE_LEN: usize = 18;

/// Truncation suffix for long tag types.
const TRUNCATION_SUFFIX: &str = "...";

/// 22x7 ASCII art renderer.
pub struct AsciiRenderer;

impl AsciiRenderer {
    /// Render a detected tag into a 22x7 frame.
    ///
    /// Each line is exactly [`DISPLAY_WIDTH`] characters wide.
    /// Returns a `Vec<String>` with exactly [`DISPLAY_HEIGHT`] entries.
    pub fn render_tag(&self, tag: &TagInfo, tap_count: u64) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);
        lines.push(Self::top_border());
        lines.push(Self::pad_line("NFC TAG DETECTED"));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line(&format!("UID {}", tag.uid)));
        lines.push(Self::pad_line(&format!("TYPE {}", Self::truncate_type(&tag.tag_type))));
        lines.push(Self::pad_line(&format!("tap #{}", tap_count)));
        lines.push(Self::bottom_border());
        lines
    }

    /// Render the idle / waiting screen when no tag is present.
    pub fn render_idle(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);
        lines.push(Self::top_border());
        lines.push(Self::pad_line("NEARFIELD"));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line("scanning..."));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line("tap a tag"));
        lines.push(Self::bottom_border());
        lines
    }

    /// Build the top border line.
    fn top_border() -> String {
        format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2))
    }

    /// Build the bottom border line.
    fn bottom_border() -> String {
        format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2))
    }

    /// Pad content to exactly `DISPLAY_WIDTH - 2` characters and wrap
    /// in vertical bars. The result is always `DISPLAY_WIDTH` chars wide.
    fn pad_line(content: &str) -> String {
        let inner = DISPLAY_WIDTH - 2;
        let truncated: String = content.chars().take(inner).collect();
        format!("|{:width$}|", truncated, width = inner)
    }

    /// Truncate a tag type string if it exceeds the maximum display width.
    fn truncate_type(tag_type: &str) -> String {
        if tag_type.len() > MAX_TAG_TYPE_LEN {
            let mut t = tag_type.chars().take(MAX_TAG_TYPE_LEN - TRUNCATION_SUFFIX.len()).collect::<String>();
            t.push_str(TRUNCATION_SUFFIX);
            t
        } else {
            tag_type.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_tag() -> TagInfo {
        TagInfo {
            uid: "7ed59290".into(),
            uid_raw: vec![0x7e, 0xd5, 0x92, 0x90],
            atqa: vec![0x00, 0x04],
            sak: 0x08,
            tag_type: "Mifare Classic 1K".into(),
            timestamp: chrono::Utc::now(),
        }
    }

    #[test]
    fn render_tag_produces_correct_number_of_lines() {
        let r = AsciiRenderer;
        assert_eq!(r.render_tag(&test_tag(), 1).len(), DISPLAY_HEIGHT);
    }

    #[test]
    fn render_tag_produces_correct_line_width() {
        let r = AsciiRenderer;
        for line in &r.render_tag(&test_tag(), 1) {
            assert_eq!(line.chars().count(), DISPLAY_WIDTH);
        }
    }

    #[test]
    fn render_idle_produces_correct_number_of_lines() {
        let r = AsciiRenderer;
        assert_eq!(r.render_idle().len(), DISPLAY_HEIGHT);
    }

    #[test]
    fn render_idle_produces_correct_line_width() {
        let r = AsciiRenderer;
        for line in &r.render_idle() {
            assert_eq!(line.chars().count(), DISPLAY_WIDTH);
        }
    }

    #[test]
    fn output_contains_tag_uid_and_count() {
        let r = AsciiRenderer;
        let lines = r.render_tag(&test_tag(), 42);
        let combined: String = lines.join("");
        assert!(combined.contains("7ed59290"));
        assert!(combined.contains("tap #42"));
    }

    #[test]
    fn truncate_long_tag_type() {
        let long_type = "Mifare Classic 4K Extended Edition";
        let truncated = AsciiRenderer::truncate_type(long_type);
        assert!(truncated.ends_with("..."));
        assert!(truncated.len() <= MAX_TAG_TYPE_LEN);
    }
}

//! # ASCII Art Renderer
//!
//! Lays out NFC tag data into a fixed 22×7 character grid, designed for
//! small e-ink displays (e.g. Waveshare 2.13" — 250×122 px).
//!
//! The 22-column width fits on a 250 px wide display at ~11 px per
//! monospace character.
//!
//! ## Rendered Layout
//!
//! ```text
//! ┌────────────────────┐
//! │ ⚡ NFC TAG DETECTED │
//! │                    │
//! │ UID 7ed59290       │
//! │ TYPE Mifare Classic│
//! │ TIME 14:32:01      │
//! │                    │
//! │ [      tap #42    ]│
//! └────────────────────┘
//! ```
//!
//! ## Why ASCII Art?
//!
//! - **Universal** — works on any display (e-ink, OLED, TFT, terminal).
//! - **Low bandwidth** — 22×7 = 154 bytes per frame.
//! - **Simple** — no font rendering, no image libraries.
//! - **Cool aesthetic** — retro terminal vibe.

use crate::nfc::TagInfo;

/// Width of the display grid in characters.
pub const DISPLAY_WIDTH: usize = 22;

/// Height of the display grid in characters.
pub const DISPLAY_HEIGHT: usize = 7;

/// The 22×7 ASCII art renderer.
pub struct AsciiRenderer;

impl AsciiRenderer {
    /// Render a `TagInfo` into a 22×7 grid.
    ///
    /// Each line is exactly 22 characters wide (padded with spaces).
    /// Returns a `Vec<String>` with exactly 7 entries.
    pub fn render_tag(&self, tag: &TagInfo, tap_count: u64) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);

        // Row 0: Top border
        lines.push(format!("┌{}┐", "─".repeat(DISPLAY_WIDTH - 2)));

        // Row 1: Title
        let title = "NFC TAG";
        lines.push(format!("│ {:<width$}│", title, width = DISPLAY_WIDTH - 3));

        // Row 2: Empty spacer
        lines.push(format!("│{empty}│", empty = " ".repeat(DISPLAY_WIDTH - 2)));

        // Row 3: UID
        let uid_line = format!("UID {}", tag.uid);
        lines.push(Self::pad_line(&uid_line));

        // Row 4: Tag type (truncated if longer than 18 chars)
        let mut tag_type = tag.tag_type.clone();
        if tag_type.len() > 18 {
            tag_type.truncate(15);
            tag_type.push_str("...");
        }
        let type_line = format!("TYPE {}", tag_type);
        lines.push(Self::pad_line(&type_line));

        // Row 5: Tap count
        let count_line = format!("tap #{}", tap_count);
        lines.push(Self::pad_line(&count_line));

        // Row 6: Bottom border
        lines.push(format!("└{}┘", "─".repeat(DISPLAY_WIDTH - 2)));

        debug_assert_eq!(lines.len(), DISPLAY_HEIGHT);
        lines
    }

    /// Render an "idle / waiting" screen when no tag is present.
    pub fn render_idle(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);

        lines.push(format!("┌{}┐", "─".repeat(DISPLAY_WIDTH - 2)));
        lines.push(format!("│ {:<width$}│", "NEARFIELD", width = DISPLAY_WIDTH - 3));
        lines.push(format!("│{empty}│", empty = " ".repeat(DISPLAY_WIDTH - 2)));
        lines.push(format!("│ {:<width$}│", "scanning...", width = DISPLAY_WIDTH - 3));
        lines.push(format!("│{empty}│", empty = " ".repeat(DISPLAY_WIDTH - 2)));
        lines.push(format!("│ {:<width$}│", "tap a tag", width = DISPLAY_WIDTH - 3));
        lines.push(format!("└{}┘", "─".repeat(DISPLAY_WIDTH - 2)));

        lines
    }

    /// Render a recent history table (top 3 entries).
    ///
    /// Useful for the idle screen between reads.
    pub fn render_history(&self, entries: &[crate::storage::NfcLogEntry]) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);
        lines.push(format!("┌{}┐", "─".repeat(DISPLAY_WIDTH - 2)));
        lines.push(format!("│ {:<width$}│", "RECENT", width = DISPLAY_WIDTH - 3));

        let display_count = std::cmp::min(entries.len(), 4);
        for i in 0..display_count {
            let entry = &entries[i];
            let uid_short = if entry.uid.len() > 8 {
                &entry.uid[..8]
            } else {
                &entry.uid
            };
            let line = format!(" {} {}", uid_short, entry.tag_type.as_deref().unwrap_or("?"));
            lines.push(Self::pad_line(&line));
        }

        // Fill remaining rows with blanks.
        while lines.len() < DISPLAY_HEIGHT - 1 {
            lines.push(format!("│{empty}│", empty = " ".repeat(DISPLAY_WIDTH - 2)));
        }

        lines.push(format!("└{}┘", "─".repeat(DISPLAY_WIDTH - 2)));
        lines
    }

    /// Pad content to exactly `DISPLAY_WIDTH - 2` chars between the vertical bars,
    /// then wrap with `│...│`. Result is always `DISPLAY_WIDTH` chars wide.
    fn pad_line(content: &str) -> String {
        let inner_width = DISPLAY_WIDTH - 2; // 20 chars between the vertical bars
        let truncated: String = content.chars().take(inner_width).collect();
        format!("│{:width$}│", truncated, width = inner_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfc::TagInfo;

    fn make_test_tag() -> TagInfo {
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
    fn test_render_tag_output_size() {
        let renderer = AsciiRenderer;
        let tag = make_test_tag();
        let lines = renderer.render_tag(&tag, 1);

        assert_eq!(lines.len(), DISPLAY_HEIGHT);
        for line in &lines {
            assert_eq!(
                line.chars().count(),
                DISPLAY_WIDTH,
                "Line {line:?} is not {DISPLAY_WIDTH} chars"
            );
        }
    }

    #[test]
    fn test_render_idle_output_size() {
        let renderer = AsciiRenderer;
        let lines = renderer.render_idle();

        assert_eq!(lines.len(), DISPLAY_HEIGHT);
        for line in &lines {
            assert_eq!(line.chars().count(), DISPLAY_WIDTH);
        }
    }

    #[test]
    fn test_render_history_output_size() {
        let renderer = AsciiRenderer;
        let lines = renderer.render_history(&[]);

        assert_eq!(lines.len(), DISPLAY_HEIGHT);
        for line in &lines {
            assert_eq!(line.chars().count(), DISPLAY_WIDTH);
        }
    }

    #[test]
    fn test_tag_uid_in_output() {
        let renderer = AsciiRenderer;
        let tag = make_test_tag();
        let lines = renderer.render_tag(&tag, 42);

        let combined: String = lines.join("");
        assert!(combined.contains("7ed59290"), "UID missing from output");
        assert!(combined.contains("tap #42"), "tap count missing");
    }
}

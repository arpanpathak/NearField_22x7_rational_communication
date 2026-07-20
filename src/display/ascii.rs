use crate::nfc::TagInfo;

pub const DISPLAY_WIDTH: usize = 22;
pub const DISPLAY_HEIGHT: usize = 7;

pub struct AsciiRenderer;

impl AsciiRenderer {
    pub fn render_tag(&self, tag: &TagInfo, tap_count: u64) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);
        lines.push(format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2)));
        lines.push(Self::pad_line("NFC TAG DETECTED"));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line(&format!("UID {}", tag.uid)));
        let mut tag_type = tag.tag_type.clone();
        if tag_type.len() > 18 {
            tag_type.truncate(15);
            tag_type.push_str("...");
        }
        lines.push(Self::pad_line(&format!("TYPE {}", tag_type)));
        lines.push(Self::pad_line(&format!("tap #{}", tap_count)));
        lines.push(format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2)));
        lines
    }

    pub fn render_idle(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(DISPLAY_HEIGHT);
        lines.push(format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2)));
        lines.push(Self::pad_line("NEARFIELD"));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line("scanning..."));
        lines.push(Self::pad_line(""));
        lines.push(Self::pad_line("tap a tag"));
        lines.push(format!("+{}+", "-".repeat(DISPLAY_WIDTH - 2)));
        lines
    }

    fn pad_line(content: &str) -> String {
        let inner_width = DISPLAY_WIDTH - 2;
        let truncated: String = content.chars().take(inner_width).collect();
        format!("|{:width$}|", truncated, width = inner_width)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert_eq!(line.chars().count(), DISPLAY_WIDTH);
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
    fn test_tag_uid_in_output() {
        let renderer = AsciiRenderer;
        let tag = make_test_tag();
        let lines = renderer.render_tag(&tag, 42);
        let combined: String = lines.join("");
        assert!(combined.contains("7ed59290"));
        assert!(combined.contains("tap #42"));
    }
}

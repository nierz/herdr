use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub(crate) fn display_width(text: &str) -> usize {
    UnicodeWidthStr::width(text)
}

pub(crate) fn display_width_u16(text: &str) -> u16 {
    display_width(text).min(u16::MAX as usize) as u16
}

pub(crate) fn truncate_end(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }
    if max_width == 1 {
        return "…".to_string();
    }

    let prefix = take_prefix_width(text, max_width.saturating_sub(1));
    format!("{prefix}…")
}

/// Splits `text` into a head that fits within `max_width` display columns and
/// the remaining tail, preferring a whitespace boundary and falling back to a
/// hard split for a single unbroken word. The tail is empty when `text` fits.
pub(crate) fn split_at_width(text: &str, max_width: usize) -> (&str, &str) {
    if display_width(text) <= max_width {
        return (text, "");
    }
    if max_width == 0 {
        return ("", text);
    }

    let mut width = 0usize;
    let mut hard_end = 0usize;
    let mut break_at = None;
    for (index, ch) in text.char_indices() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        if ch.is_whitespace() && index > 0 {
            break_at = Some(index);
        }
        width += ch_width;
        hard_end = index + ch.len_utf8();
    }

    if text[hard_end..].starts_with(char::is_whitespace) {
        break_at = Some(hard_end);
    }
    if let Some(index) = break_at {
        let head = text[..index].trim_end();
        let tail = text[index..].trim_start();
        if !head.is_empty() && !tail.is_empty() {
            return (head, tail);
        }
    }

    (&text[..hard_end], text[hard_end..].trim_start())
}

pub(crate) fn middle_elide(text: &str, max_width: usize) -> String {
    if display_width(text) <= max_width {
        return text.to_string();
    }
    if max_width <= 1 {
        return "…".to_string();
    }

    let content_width = max_width.saturating_sub(1);
    let left_width = content_width / 2;
    let right_width = content_width.saturating_sub(left_width);
    let prefix = take_prefix_width(text, left_width);
    let suffix = take_suffix_width(text, right_width);
    format!("{prefix}…{suffix}")
}

fn take_prefix_width(text: &str, max_width: usize) -> String {
    let mut output = String::new();
    let mut width = 0usize;
    for ch in text.chars() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        output.push(ch);
        width += ch_width;
    }
    output
}

fn take_suffix_width(text: &str, max_width: usize) -> String {
    let mut output = Vec::new();
    let mut width = 0usize;
    for ch in text.chars().rev() {
        let ch_width = UnicodeWidthChar::width(ch).unwrap_or(0);
        if width + ch_width > max_width {
            break;
        }
        output.push(ch);
        width += ch_width;
    }
    output.into_iter().rev().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_end_uses_display_width() {
        let text = truncate_end("提交 herdr 的反馈", 16);

        assert_eq!(text, "提交 herdr 的反…");
        assert!(display_width(&text) <= 16);
    }

    #[test]
    fn split_at_width_prefers_whitespace_and_respects_display_width() {
        assert_eq!(split_at_width("short title", 20), ("short title", ""));
        assert_eq!(
            split_at_width("Investigate internal activities", 23),
            ("Investigate internal", "activities")
        );
        assert_eq!(
            split_at_width("unbreakablewordthatkeepsgoing", 10),
            ("unbreakabl", "ewordthatkeepsgoing")
        );

        let (head, tail) = split_at_width("修复标题很长的任务", 7);
        assert!(display_width(head) <= 7);
        assert_eq!(format!("{head}{tail}"), "修复标题很长的任务");
    }

    #[test]
    fn middle_elide_uses_display_width() {
        let text = middle_elide("重构用户认证模块并迁移到统一登录服务", 12);

        assert!(text.contains('…'));
        assert!(display_width(&text) <= 12);
    }
}

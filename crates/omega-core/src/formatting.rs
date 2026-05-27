//! Telegram HTML formatting engine.
//!
//! Converts markdown-style text into Telegram-compatible HTML with
//! blockquote cards, code blocks, tables, progress bars, and
//! proper HTML entity escaping.

use std::fmt::Write;

/// Escape HTML special characters for Telegram's HTML parse mode.
pub fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Convert markdown-style text to Telegram HTML.
///
/// Handles: **bold**, `inline code`, ```code blocks```,
/// > blockquotes, # headings, - list items, [links](url).
pub fn markdown_to_telegram_html(md: &str) -> String {
    let mut out = String::with_capacity(md.len() * 2);
    let mut in_code_block = false;
    let mut code_block_buf = String::new();

    for line in md.lines() {
        let trimmed = line.trim();

        // Fenced code blocks
        if trimmed.starts_with("```") {
            if in_code_block {
                out.push_str("<pre>");
                out.push_str(&escape_html(&code_block_buf));
                out.push_str("</pre>\n");
                code_block_buf.clear();
                in_code_block = false;
            } else {
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            if !code_block_buf.is_empty() {
                code_block_buf.push('\n');
            }
            code_block_buf.push_str(line);
            continue;
        }

        // Headings
        if trimmed.starts_with("# ") {
            let heading = &trimmed[2..];
            let _ = write!(out, "\n<b>{}</b>\n", escape_html(heading));
            continue;
        }
        if trimmed.starts_with("## ") {
            let heading = &trimmed[3..];
            let _ = write!(out, "\n<b>{}</b>\n", escape_html(heading));
            continue;
        }
        if trimmed.starts_with("### ") {
            let heading = &trimmed[4..];
            let _ = write!(out, "<b>{}</b>\n", escape_html(heading));
            continue;
        }

        // Blockquotes
        if trimmed.starts_with("> ") {
            let content = &trimmed[2..];
            let _ = write!(out, "<blockquote>{}</blockquote>\n", format_inline(&escape_html(content)));
            continue;
        }

        // Horizontal rules
        if trimmed == "---" || trimmed == "━━━" || trimmed.chars().all(|c| c == '─' || c == '━') {
            out.push_str("━━━━━━━━━━\n");
            continue;
        }

        // List items
        if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
            let item = &trimmed[2..];
            let _ = write!(out, "• {}\n", format_inline(&escape_html(item)));
            continue;
        }

        // Numbered list items
        if trimmed.len() > 2 {
            let first_dot = trimmed.find(". ");
            if let Some(pos) = first_dot {
                if pos <= 3 && trimmed[..pos].chars().all(|c| c.is_ascii_digit()) {
                    let item = &trimmed[pos + 2..];
                    let num = &trimmed[..pos];
                    let _ = write!(out, "{}. {}\n", num, format_inline(&escape_html(item)));
                    continue;
                }
            }
        }

        // Empty line
        if trimmed.is_empty() {
            out.push('\n');
            continue;
        }

        // Regular text with inline formatting
        let _ = write!(out, "{}\n", format_inline(&escape_html(trimmed)));
    }

    // Unclosed code block
    if in_code_block && !code_block_buf.is_empty() {
        out.push_str("<pre>");
        out.push_str(&escape_html(&code_block_buf));
        out.push_str("</pre>\n");
    }

    collapse_blank_lines(&out)
}

/// Apply inline formatting: **bold**, *italic*, `code`, [link](url).
fn format_inline(text: &str) -> String {
    let mut result = text.to_string();

    // Bold: **text**
    while let Some(start) = result.find("**") {
        if let Some(end) = result[start + 2..].find("**") {
            let end = start + 2 + end;
            let inner = result[start + 2..end].to_string();
            result = format!("{}<b>{}</b>{}", &result[..start], inner, &result[end + 2..]);
        } else {
            break;
        }
    }

    // Italic: *text* (but not inside bold tags)
    let mut buf = String::with_capacity(result.len());
    let mut chars = result.chars().peekable();
    let mut in_tag = false;
    while let Some(c) = chars.next() {
        if c == '<' {
            in_tag = true;
            buf.push(c);
        } else if c == '>' {
            in_tag = false;
            buf.push(c);
        } else if c == '*' && !in_tag {
            if let Some(end_pos) = find_closing_char(&result[buf.len() + 1..], '*') {
                let inner: String = chars.by_ref().take(end_pos).collect();
                buf.push_str("<i>");
                buf.push_str(&inner);
                buf.push_str("</i>");
                chars.next(); // consume closing *
            } else {
                buf.push(c);
            }
        } else {
            buf.push(c);
        }
    }
    result = buf;

    // Inline code: `text`
    while let Some(start) = result.find('`') {
        if let Some(end) = result[start + 1..].find('`') {
            let end = start + 1 + end;
            let inner = result[start + 1..end].to_string();
            result = format!("{}<code>{}</code>{}", &result[..start], inner, &result[end + 1..]);
        } else {
            break;
        }
    }

    result
}

fn find_closing_char(text: &str, ch: char) -> Option<usize> {
    text.find(ch)
}

fn collapse_blank_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut blank_count = 0;
    for line in text.lines() {
        if line.trim().is_empty() {
            blank_count += 1;
            if blank_count <= 1 {
                out.push('\n');
            }
        } else {
            blank_count = 0;
            out.push_str(line);
            out.push('\n');
        }
    }
    out.trim_end().to_string()
}

/// Format a blockquote card with title and body (AISB report style).
pub fn blockquote_card(title: &str, body: &str) -> String {
    let mut out = String::new();
    let _ = write!(out, "<blockquote><b>{}</b>\n{}</blockquote>", escape_html(title), escape_html(body));
    out
}

/// Format a progress bar for Telegram.
/// `progress` is 0.0..=1.0, `width` is number of characters.
pub fn progress_bar(progress: f32, width: usize) -> String {
    let clamped = progress.clamp(0.0, 1.0);
    let filled = (clamped * width as f32).round() as usize;
    let empty = width.saturating_sub(filled);
    let pct = (clamped * 100.0).round() as u32;
    format!(
        "<code>[{}{}] {}%</code>",
        "█".repeat(filled),
        "░".repeat(empty),
        pct
    )
}

/// Format a status badge.
pub fn status_badge(label: &str, ok: bool) -> String {
    let icon = if ok { "🟢" } else { "🔴" };
    format!("{} <b>{}</b>", icon, escape_html(label))
}

/// Format a key-value table for Telegram (no real table support, uses aligned text).
pub fn kv_table(rows: &[(&str, &str)]) -> String {
    let mut out = String::new();
    let max_key = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, val) in rows {
        let _ = writeln!(
            out,
            "<b>{:width$}</b>  <code>{}</code>",
            escape_html(key),
            escape_html(val),
            width = max_key
        );
    }
    out.trim_end().to_string()
}

/// Format an oracle report for Telegram delivery.
pub fn format_oracle_report(project: &str, status: &str, body: &str) -> String {
    let status_icon = match status.to_uppercase().as_str() {
        "DONE" => "✅",
        "FAILED" => "🔴",
        "PENDING" => "🟡",
        _ => "📋",
    };
    let header = format!("{} <b>{}</b> — {}", status_icon, escape_html(project), escape_html(status));
    let formatted_body = markdown_to_telegram_html(body);
    format!("{}\n━━━━━━━━━━\n{}", header, formatted_body)
}

/// Format an audit score summary.
pub fn format_audit_scores(scores: &[(&str, f32, f32)]) -> String {
    let mut out = String::from("<b>Quality Arsenal Scores</b>\n━━━━━━━━━━\n");
    for (name, score, max) in scores {
        let pct = if *max > 0.0 { (score / max) * 100.0 } else { 0.0 };
        let icon = if pct >= 80.0 {
            "🟢"
        } else if pct >= 50.0 {
            "🟡"
        } else {
            "🔴"
        };
        let _ = writeln!(
            out,
            "{} <b>{}</b>  <code>{:.0}/{:.0}</code>  ({})",
            icon,
            escape_html(name),
            score,
            max,
            progress_bar(pct / 100.0, 10)
        );
    }
    out.trim_end().to_string()
}

/// Split a message into chunks that fit Telegram's 4096-char limit.
pub fn split_message(text: &str, max_len: usize) -> Vec<String> {
    if text.len() <= max_len {
        return vec![text.to_string()];
    }

    let mut chunks = Vec::new();
    let mut current = String::new();

    for line in text.lines() {
        if current.len() + line.len() + 1 > max_len {
            if !current.is_empty() {
                chunks.push(current.clone());
                current.clear();
            }
            if line.len() > max_len {
                // Force-split very long lines
                let mut remaining = line;
                while remaining.len() > max_len {
                    chunks.push(remaining[..max_len].to_string());
                    remaining = &remaining[max_len..];
                }
                current.push_str(remaining);
            } else {
                current.push_str(line);
            }
        } else {
            if !current.is_empty() {
                current.push('\n');
            }
            current.push_str(line);
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_html_chars() {
        assert_eq!(escape_html("<b>&\"test\"</b>"), "&lt;b&gt;&amp;&quot;test&quot;&lt;/b&gt;");
    }

    #[test]
    fn markdown_headings() {
        let html = markdown_to_telegram_html("# Title\n## Subtitle");
        assert!(html.contains("<b>Title</b>"));
        assert!(html.contains("<b>Subtitle</b>"));
    }

    #[test]
    fn markdown_code_block() {
        let html = markdown_to_telegram_html("```\nfn main() {}\n```");
        assert!(html.contains("<pre>"));
        assert!(html.contains("fn main()"));
    }

    #[test]
    fn markdown_blockquote() {
        let html = markdown_to_telegram_html("> Important note");
        assert!(html.contains("<blockquote>"));
    }

    #[test]
    fn markdown_list_items() {
        let html = markdown_to_telegram_html("- First\n- Second");
        assert!(html.contains("• First"));
        assert!(html.contains("• Second"));
    }

    #[test]
    fn progress_bar_formatting() {
        let bar = progress_bar(0.5, 10);
        assert!(bar.contains("50%"));
        assert!(bar.contains("█████░░░░░"));
    }

    #[test]
    fn progress_bar_clamping() {
        let bar = progress_bar(1.5, 10);
        assert!(bar.contains("100%"));
        let bar0 = progress_bar(-0.5, 10);
        assert!(bar0.contains("0%"));
    }

    #[test]
    fn split_message_short() {
        let chunks = split_message("hello", 4096);
        assert_eq!(chunks.len(), 1);
    }

    #[test]
    fn split_message_long() {
        let text = "a\n".repeat(3000);
        let chunks = split_message(&text, 4096);
        assert!(chunks.len() > 1);
        for chunk in &chunks {
            assert!(chunk.len() <= 4096);
        }
    }

    #[test]
    fn blockquote_card_format() {
        let card = blockquote_card("Status", "All good");
        assert!(card.contains("<blockquote>"));
        assert!(card.contains("<b>Status</b>"));
    }

    #[test]
    fn status_badge_ok_and_fail() {
        assert!(status_badge("Build", true).contains("🟢"));
        assert!(status_badge("Build", false).contains("🔴"));
    }

    #[test]
    fn format_oracle_report_done() {
        let report = format_oracle_report("Causio", "DONE", "Fixed auth flow");
        assert!(report.contains("✅"));
        assert!(report.contains("Causio"));
    }

    #[test]
    fn format_audit_scores_display() {
        let scores = vec![
            ("Code", 336.0_f32, 420.0_f32),
            ("Debug", 100.0, 360.0),
        ];
        let output = format_audit_scores(&scores);
        assert!(output.contains("🟢")); // 336/420 = 80%
        assert!(output.contains("🔴")); // 100/360 = 27.8%
        assert!(output.contains("Code"));
    }

    #[test]
    fn kv_table_format() {
        let table = kv_table(&[("Status", "Online"), ("Uptime", "12h")]);
        assert!(table.contains("Status"));
        assert!(table.contains("Online"));
    }
}

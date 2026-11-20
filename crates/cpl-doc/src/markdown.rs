//! The Markdown doc comments are written in: paragraphs, `#` headings,
//! `-` lists, fenced code and inline `code`, `*emphasis*` and
//! `[links](url)`. Anything else is text.

use crate::highlight::{escape, highlight};

/// Inline markup of one run of text.
fn inline(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(at) = rest.find(['`', '*', '[']) {
        out.push_str(&escape(&rest[..at]));
        let tail = &rest[at..];
        let marker = tail.as_bytes()[0];
        let consumed = match marker {
            b'`' => tail[1..].find('`').map(|end| {
                out.push_str("<code>");
                out.push_str(&escape(&tail[1..=end]));
                out.push_str("</code>");
                end + 2
            }),
            b'*' => {
                let (tag, width) = if tail.starts_with("**") {
                    ("strong", 2)
                } else {
                    ("em", 1)
                };
                let close = &tail[..width];
                tail[width..]
                    .find(close)
                    .filter(|end| *end > 0 && !tail[width..].starts_with(' '))
                    .map(|end| {
                        out.push_str(&format!("<{tag}>"));
                        out.push_str(&inline(&tail[width..width + end]));
                        out.push_str(&format!("</{tag}>"));
                        end + 2 * width
                    })
            }
            _ => tail.find("](").and_then(|middle| {
                let end = tail[middle..].find(')')? + middle;
                let (label, url) = (&tail[1..middle], &tail[middle + 2..end]);
                if label.contains('[') || url.contains(char::is_whitespace) {
                    return None;
                }
                out.push_str(&format!("<a href=\"{}\">", escape(url)));
                out.push_str(&inline(label));
                out.push_str("</a>");
                Some(end + 1)
            }),
        };
        match consumed {
            Some(consumed) => rest = &tail[consumed..],
            None => {
                out.push_str(&escape(&tail[..1]));
                rest = &tail[1..];
            }
        }
    }
    out.push_str(&escape(rest));
    out
}

/// Renders a doc comment.
pub(crate) fn render(doc: &str) -> String {
    let mut out = String::new();
    let mut paragraph: Vec<&str> = Vec::new();
    let mut list: Vec<String> = Vec::new();
    let mut fence: Option<(String, Vec<&str>)> = None;

    fn flush(out: &mut String, paragraph: &mut Vec<&str>, list: &mut Vec<String>) {
        if !paragraph.is_empty() {
            out.push_str("<p>");
            out.push_str(&inline(&paragraph.join(" ")));
            out.push_str("</p>\n");
            paragraph.clear();
        }
        if !list.is_empty() {
            out.push_str("<ul>\n");
            for item in list.drain(..) {
                out.push_str("<li>");
                out.push_str(&inline(&item));
                out.push_str("</li>\n");
            }
            out.push_str("</ul>\n");
        }
    }

    for line in doc.lines() {
        if let Some((language, code)) = &mut fence {
            if line.trim_start().starts_with("```") {
                let text = code.join("\n");
                let body = if matches!(language.as_str(), "" | "cp") {
                    highlight(&text)
                } else {
                    escape(&text)
                };
                out.push_str("<pre><code>");
                out.push_str(&body);
                out.push_str("</code></pre>\n");
                fence = None;
            } else {
                code.push(line);
            }
            continue;
        }
        let trimmed = line.trim();
        if let Some(language) = trimmed.strip_prefix("```") {
            flush(&mut out, &mut paragraph, &mut list);
            fence = Some((language.trim().to_string(), Vec::new()));
        } else if trimmed.is_empty() {
            flush(&mut out, &mut paragraph, &mut list);
        } else if let Some(heading) = trimmed.strip_prefix('#') {
            flush(&mut out, &mut paragraph, &mut list);
            // Headings in a comment sit under the item's own heading.
            let text = heading.trim_start_matches('#').trim();
            out.push_str(&format!("<h4>{}</h4>\n", inline(text)));
        } else if let Some(item) = trimmed.strip_prefix("- ") {
            if !paragraph.is_empty() {
                flush(&mut out, &mut paragraph, &mut list);
            }
            list.push(item.to_string());
        } else if !list.is_empty() && line.starts_with(' ') {
            // A list item continued on the next line.
            let last = list.last_mut().unwrap();
            last.push(' ');
            last.push_str(trimmed);
        } else {
            if !list.is_empty() {
                flush(&mut out, &mut paragraph, &mut list);
            }
            paragraph.push(trimmed);
        }
    }
    // An unclosed fence is still code.
    if let Some((_, code)) = fence {
        out.push_str("<pre><code>");
        out.push_str(&escape(&code.join("\n")));
        out.push_str("</code></pre>\n");
    }
    flush(&mut out, &mut paragraph, &mut list);
    out
}

/// One line of a doc comment as inline HTML, for summaries.
pub(crate) fn render_inline(text: &str) -> String {
    inline(&text.split_whitespace().collect::<Vec<_>>().join(" "))
}

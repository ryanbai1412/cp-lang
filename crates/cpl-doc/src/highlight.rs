//! CP Lang source as HTML, colored by token.

use cpl_syntax::TokenKind;

/// `text` with `&`, `<`, `>` and `"` escaped.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            c => out.push(c),
        }
    }
    out
}

/// The CSS class of a token, when it has one.
fn class(kind: TokenKind, text: &str) -> Option<&'static str> {
    match kind {
        TokenKind::Int | TokenKind::Float => Some("n"),
        TokenKind::Str | TokenKind::RawStr | TokenKind::Char => Some("s"),
        TokenKind::Label => Some("k"),
        TokenKind::Ident => text
            .starts_with(|c: char| c.is_ascii_uppercase())
            .then_some("t"),
        _ if TokenKind::keyword(text).is_some() => Some("k"),
        _ => None,
    }
}

/// Highlights `code`. Whatever is between tokens (spaces, comments) is
/// kept as written, so the result reads the same as the source.
pub(crate) fn highlight(code: &str) -> String {
    let lexed = cpl_syntax::lex(code);
    let mut out = String::with_capacity(code.len() * 2);
    let mut at = 0usize;
    let gap = |out: &mut String, text: &str| {
        // Comments are the only non-blank text between tokens.
        match text.find("//") {
            Some(start) => {
                out.push_str(&escape(&text[..start]));
                for (i, line) in text[start..].split('\n').enumerate() {
                    if i > 0 {
                        out.push('\n');
                    }
                    match line.find("//") {
                        Some(slash) => {
                            out.push_str(&escape(&line[..slash]));
                            out.push_str("<span class=\"c\">");
                            out.push_str(&escape(&line[slash..]));
                            out.push_str("</span>");
                        }
                        None => out.push_str(&escape(line)),
                    }
                }
            }
            None => out.push_str(&escape(text)),
        }
    };
    for token in &lexed.tokens {
        let (start, end) = (token.span.start as usize, token.span.end as usize);
        // Layout tokens are empty or cover text an earlier token did.
        if start < at || end <= start || end > code.len() {
            continue;
        }
        let text = &code[start..end];
        if text.trim().is_empty() {
            continue;
        }
        gap(&mut out, &code[at..start]);
        match class(token.kind, text) {
            Some(class) => {
                out.push_str("<span class=\"");
                out.push_str(class);
                out.push_str("\">");
                out.push_str(&escape(text));
                out.push_str("</span>");
            }
            None => out.push_str(&escape(text)),
        }
        at = end;
    }
    gap(&mut out, &code[at..]);
    out
}

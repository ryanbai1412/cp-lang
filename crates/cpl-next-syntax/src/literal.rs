//! Values of literals.

use cpl_diag::Span;

/// The value of an integer literal: `42`, `1_000_000`, `0xff`, `0b101`, and
/// `1e9` or `2.5e3`-style literals whose value is an exact integer.
pub fn int_value(text: &str) -> Option<i128> {
    let clean: String = text.chars().filter(|c| *c != '_').collect();
    let lower = clean.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix("0x") {
        return i128::from_str_radix(hex, 16).ok();
    }
    if let Some(bin) = lower.strip_prefix("0b") {
        return i128::from_str_radix(bin, 2).ok();
    }
    let (mantissa, exp) = match lower.split_once('e') {
        Some((m, e)) => (m, e.parse::<i32>().ok()?),
        None => (lower.as_str(), 0),
    };
    let (whole, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let frac = frac.trim_end_matches('0');
    let exp = exp - frac.len() as i32;
    let digits: i128 = format!("{whole}{frac}").parse().ok()?;
    if exp >= 0 {
        10i128.checked_pow(exp as u32)?.checked_mul(digits)
    } else {
        let div = 10i128.checked_pow(exp.unsigned_abs())?;
        (digits % div == 0).then(|| digits / div)
    }
}

/// One piece of an interpolating string.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StrPart {
    /// Literal text (escapes not yet processed).
    Text(Span),
    /// The inside of `{...}`: an expression to parse.
    Expr(Span),
}

/// Splits the token text of an interpolating string (quotes included) into
/// text and `{expr}` parts. Spans are relative to `text`.
pub fn string_parts(text: &str) -> Vec<StrPart> {
    let bytes = text.as_bytes();
    let end = if bytes.len() >= 2 && bytes[bytes.len() - 1] == b'"' {
        bytes.len() - 1
    } else {
        bytes.len()
    };
    let mut parts = Vec::new();
    let mut i = 1;
    let mut text_start = 1;
    while i < end {
        match bytes[i] {
            b'\\' => i += 2,
            b'{' if bytes.get(i + 1) == Some(&b'{') => i += 2,
            b'}' if bytes.get(i + 1) == Some(&b'}') => i += 2,
            b'{' => {
                if i > text_start {
                    parts.push(StrPart::Text(Span::new(text_start as u32, i as u32)));
                }
                let expr_start = i + 1;
                let mut depth = 1;
                let mut in_str = false;
                i += 1;
                while i < end && depth > 0 {
                    match bytes[i] {
                        b'\\' if in_str => i += 1,
                        b'"' => in_str = !in_str,
                        b'{' if !in_str => depth += 1,
                        b'}' if !in_str => depth -= 1,
                        _ => {}
                    }
                    i += 1;
                }
                let expr_end = if depth == 0 { i - 1 } else { i.min(end) };
                parts.push(StrPart::Expr(Span::new(expr_start as u32, expr_end as u32)));
                text_start = i.min(end);
            }
            _ => i += 1,
        }
    }
    if end > text_start {
        parts.push(StrPart::Text(Span::new(text_start as u32, end as u32)));
    }
    parts
}

/// The text of a string or char literal with its escapes decoded: `\n`,
/// `\t`, `\r`, `\0`, `\xHH`, and `\c` for any other `c`; `{{` and `}}` are
/// one brace.
pub fn unescape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '\\' => match chars.next() {
                Some('n') => out.push('\n'),
                Some('t') => out.push('\t'),
                Some('r') => out.push('\r'),
                Some('0') => out.push('\0'),
                Some('x') => {
                    let hex: String = chars.clone().take(2).collect();
                    match u8::from_str_radix(&hex, 16) {
                        Ok(b) if hex.len() == 2 => {
                            chars.nth(1);
                            out.push(char::from(b));
                        }
                        _ => out.push('x'),
                    }
                }
                Some(other) => out.push(other),
                None => out.push('\\'),
            },
            '{' | '}' if chars.peek() == Some(&c) => {
                chars.next();
                out.push(c);
            }
            _ => out.push(c),
        }
    }
    out
}

/// The value of a char literal, quotes included: `'a'`, `'\n'`, `'\x41'`.
/// `None` when the text is not one quoted, decoded char.
pub fn char_value(text: &str) -> Option<char> {
    let inner = text.strip_prefix('\'')?.strip_suffix('\'')?;
    let decoded = unescape(inner);
    let mut chars = decoded.chars();
    let c = chars.next()?;
    chars.next().is_none().then_some(c)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn char_literals_decode_as_the_old_parser_did() {
        assert_eq!(char_value("'a'"), Some('a'));
        assert_eq!(char_value("'\\n'"), Some('\n'));
        assert_eq!(char_value("'\\0'"), Some('\0'));
        assert_eq!(char_value("'\\x41'"), Some('A'));
        assert_eq!(char_value("'\\''"), Some('\''));
        assert_eq!(char_value("'\\\\'"), Some('\\'));
        assert_eq!(char_value("'\\q'"), Some('q'));
        assert_eq!(char_value("'ab'"), None);
        assert_eq!(char_value("''"), None);
        assert_eq!(char_value("'a"), None);
        assert_eq!(unescape("a{{b}}\\tc"), "a{b}\tc");
    }

    #[test]
    fn integer_literals() {
        assert_eq!(int_value("42"), Some(42));
        assert_eq!(int_value("1_000_000"), Some(1_000_000));
        assert_eq!(int_value("1e9"), Some(1_000_000_000));
        assert_eq!(int_value("2e5"), Some(200_000));
        assert_eq!(int_value("0xff"), Some(255));
        assert_eq!(int_value("0b1010"), Some(10));
        assert_eq!(int_value("2.5e3"), Some(2500));
        assert_eq!(int_value("1e-3"), None);
        assert_eq!(int_value("10e-1"), Some(1));
        assert_eq!(int_value("0.5"), None);
        assert_eq!(int_value("1e999"), None);
    }

    fn parts(text: &str) -> Vec<(char, &str)> {
        string_parts(text)
            .into_iter()
            .map(|p| match p {
                StrPart::Text(s) => ('t', s.text(text)),
                StrPart::Expr(s) => ('e', s.text(text)),
            })
            .collect()
    }

    #[test]
    fn strings_split_into_text_and_expressions() {
        assert_eq!(
            parts(r#""Case #{t}: {ans + 1}""#),
            vec![('t', "Case #"), ('e', "t"), ('t', ": "), ('e', "ans + 1")]
        );
        assert_eq!(parts(r#""plain""#), vec![('t', "plain")]);
        assert_eq!(parts(r#""""#), vec![]);
    }

    #[test]
    fn doubled_braces_and_nested_strings_are_not_interpolations() {
        assert_eq!(parts(r#""{{x}} {y}""#), vec![('t', "{{x}} "), ('e', "y")]);
        assert_eq!(parts(r#""{m["a}"]}""#), vec![('e', r#"m["a}"]"#)]);
    }

    #[test]
    fn an_unclosed_interpolation_runs_to_the_end() {
        assert_eq!(parts(r#""a {b + "#), vec![('t', "a "), ('e', "b + ")]);
    }
}

/// A number format: `W` (least width), `0W` (zero-filled), then `.D` or
/// `.Df` (digits after the point), `x` (hex) or `b` (binary).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct FmtSpec {
    pub zero: bool,
    pub width: u32,
    pub kind: FmtKind,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FmtKind {
    Dec,
    Hex,
    Bin,
    Fixed(u32),
}

impl std::fmt::Display for FmtSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.zero { ":0" } else { ":" })?;
        if self.width > 0 {
            write!(f, "{}", self.width)?;
        }
        match self.kind {
            FmtKind::Dec => Ok(()),
            FmtKind::Hex => f.write_str("x"),
            FmtKind::Bin => f.write_str("b"),
            FmtKind::Fixed(places) => write!(f, ".{places}f"),
        }
    }
}

impl FmtSpec {
    /// The format `text` spells (what follows the `:`), if it is one.
    pub fn parse(text: &str) -> Option<FmtSpec> {
        let digits = |s: &str| s.len() - s.trim_start_matches(|c: char| c.is_ascii_digit()).len();
        let zero = text.starts_with('0') && text.len() > 1 && text.as_bytes()[1].is_ascii_digit();
        let rest = if zero { &text[1..] } else { text };
        let (width, rest) = rest.split_at(digits(rest));
        let kind = match rest {
            "" if !width.is_empty() => FmtKind::Dec,
            "x" => FmtKind::Hex,
            "b" => FmtKind::Bin,
            _ => {
                let places = rest.strip_prefix('.')?;
                let places = places.strip_suffix('f').unwrap_or(places);
                if places.is_empty() || digits(places) != places.len() {
                    return None;
                }
                FmtKind::Fixed(places.parse().ok().filter(|d| *d <= 300)?)
            }
        };
        let width = if width.is_empty() {
            0
        } else {
            width.parse().ok().filter(|w| *w <= 4096)?
        };
        Some(FmtSpec { zero, width, kind })
    }
}

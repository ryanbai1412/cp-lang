//! Doc text for places without Markdown links or math: LSP hover and the
//! terminal. Math written `\( O(n \log n) \)` becomes `O(n log n)`, and an
//! intra-doc link [`up`] or [`text`](Rooted.up) becomes the code `up`.

/// `doc` with its math in plain Unicode and its intra-doc links as code.
pub fn text(doc: &str) -> String {
    let mut out = String::with_capacity(doc.len());
    let mut rest = doc;
    while let Some(at) = rest.find("\\(") {
        out.push_str(&links(&rest[..at]));
        match rest[at + 2..].find("\\)") {
            Some(end) => {
                out.push_str(&math(&rest[at + 2..at + 2 + end]));
                rest = &rest[at + 2 + end + 2..];
            }
            None => {
                out.push_str(&rest[at..]);
                rest = "";
            }
        }
    }
    out.push_str(&links(rest));
    out
}

fn links(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find("[`") {
        out.push_str(&rest[..at]);
        let tail = &rest[at + 1..];
        match tail[1..].find("`]") {
            Some(end) => {
                out.push_str(&tail[..end + 2]);
                rest = &tail[end + 3..];
                if let Some((path, after)) = rest.strip_prefix('(').and_then(|r| r.split_once(')'))
                {
                    if path
                        .chars()
                        .all(|c| c.is_alphanumeric() || c == '_' || c == '.')
                    {
                        rest = after;
                    }
                }
            }
            None => {
                out.push_str(&rest[at..]);
                rest = "";
            }
        }
    }
    out.push_str(rest);
    out
}

/// LaTeX math as plain text: commands to their symbols, `^2` to `²`.
pub fn math(tex: &str) -> String {
    const WORDS: [(&str, &str); 14] = [
        ("\\log", "log"),
        ("\\ln", "ln"),
        ("\\min", "min"),
        ("\\max", "max"),
        ("\\cdot", "·"),
        ("\\times", "×"),
        ("\\alpha", "α"),
        ("\\Theta", "Θ"),
        ("\\Omega", "Ω"),
        ("\\le", "≤"),
        ("\\ge", "≥"),
        ("\\infty", "∞"),
        ("\\,", " "),
        ("\\;", " "),
    ];
    let mut s = tex.trim().to_string();
    while let Some(at) = s.find("\\sqrt{") {
        let end = s[at..].find('}').map_or(s.len(), |e| at + e);
        let inner = s[at + 6..end].to_string();
        s.replace_range(at..(end + 1).min(s.len()), &format!("√{inner}"));
    }
    for (from, to) in WORDS {
        s = s.replace(from, to);
    }
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '^' {
            out.push(c);
            continue;
        }
        let exponent: String = if chars.peek() == Some(&'{') {
            chars.next();
            chars.by_ref().take_while(|&c| c != '}').collect()
        } else {
            chars.next().map(String::from).unwrap_or_default()
        };
        match exponent
            .chars()
            .map(superscript)
            .collect::<Option<String>>()
        {
            Some(sup) => out.push_str(&sup),
            None => out.push_str(&format!("^({exponent})")),
        }
    }
    out.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .replace("( ", "(")
        .replace(" )", ")")
}

fn superscript(c: char) -> Option<char> {
    Some(match c {
        '0' => '⁰',
        '1' => '¹',
        '2' => '²',
        '3' => '³',
        '4' => '⁴',
        '5' => '⁵',
        '6' => '⁶',
        '7' => '⁷',
        '8' => '⁸',
        '9' => '⁹',
        'n' => 'ⁿ',
        _ => return None,
    })
}

//! Doc comments: which lines are documentation, and what a doc says.
//!
//! The one reader of `///` text. `cpl doc`, the docs site, hover, the
//! outline and the doc-example suite all read a doc through [`DocText`],
//! so a `cost:` line or an `example:` means the same to each of them
//! (std/README.md is the standard they follow).

/// The text after `///` when `line` is a doc comment line; a `////` ruler
/// is not one.
pub fn doc_line(line: &str) -> Option<&str> {
    let body = line.trim().strip_prefix("///")?;
    if body.starts_with('/') {
        return None;
    }
    Some(body.strip_prefix(' ').unwrap_or(body))
}

/// The `///` lines directly above the line holding byte `at` of `text`,
/// past any decorators between them, joined and trimmed.
pub fn doc_above(text: &str, at: usize) -> String {
    let head = text.get(..at.min(text.len())).unwrap_or("");
    let line_start = head.rfind('\n').map_or(0, |i| i + 1);
    let mut lines = Vec::new();
    for line in head[..line_start].lines().rev() {
        match doc_line(line) {
            Some(doc) => lines.push(doc),
            None if line.trim_start().starts_with('@') => {}
            None => break,
        }
    }
    lines.reverse();
    lines.join("\n").trim().to_string()
}

/// The labelled lines of a doc, std/README.md's facts about an operation.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Facts {
    pub cost: Option<String>,
    pub traps: Option<String>,
    pub empty: Option<String>,
    pub ties: Option<String>,
    pub order: Option<String>,
}

impl Facts {
    /// The labels, in the order std/README.md lists them.
    pub const LABELS: [&'static str; 5] = ["cost", "traps", "empty", "ties", "order"];

    /// `(label, text)` for each fact the doc states, in [`Self::LABELS`] order.
    pub fn stated(&self) -> Vec<(&'static str, &str)> {
        Self::LABELS
            .into_iter()
            .zip([
                &self.cost,
                &self.traps,
                &self.empty,
                &self.ties,
                &self.order,
            ])
            .filter_map(|(label, text)| Some((label, text.as_deref()?)))
            .collect()
    }

    fn slot(&mut self, label: &str) -> Option<&mut Option<String>> {
        Some(match label {
            "cost" => &mut self.cost,
            "traps" => &mut self.traps,
            "empty" => &mut self.empty,
            "ties" => &mut self.ties,
            "order" => &mut self.order,
            _ => return None,
        })
    }
}

/// An example in a doc: a fenced ```` ``` ```` / ```` ```cp ```` block, or an
/// `example:` line, `` `setup; expr == value` `` with optional
/// `` (stdin `...`) ``, `(empty stdin)`, `(traps R0401)` or `(error E0302)` tags.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Example {
    /// The program text: a block's lines, or a line's backticked code.
    /// `None` for an `example:` line that is prose rather than code.
    pub code: Option<String>,
    /// The `example:` line as written, after the label; for a block, its code.
    pub text: String,
    /// The 0-based line of the doc the code starts on.
    pub line: u32,
    pub block: bool,
    /// Standard input as written, without a final newline.
    pub stdin: Option<String>,
    /// The diagnostic or trap code the example stops with.
    pub expect: Option<String>,
}

impl Example {
    /// Its top-level [`statements`].
    pub fn statements(&self) -> Vec<String> {
        self.code
            .as_deref()
            .map_or_else(Vec::new, |code| statements(code, self.block))
    }

    /// An `example:` line's text, after the label.
    pub fn from_line(text: &str) -> Example {
        one_line(text.trim(), 0)
    }

    /// The statements before the last, the last one's expression, and the
    /// value after its top-level ` == `, if any: `(["let x = 1"], "x + 1", Some("2"))`.
    pub fn claim(&self) -> (Vec<String>, String, Option<String>) {
        let mut setup = self.statements();
        let last = setup.pop().unwrap_or_default();
        let sides = split_top(&last, " == ");
        match sides.split_last() {
            Some((value, expr)) if !expr.is_empty() => {
                (setup, expr.join(" == "), Some(value.to_string()))
            }
            _ => (setup, last, None),
        }
    }
}

/// The top-level statements of example code: `;`-separated on a line; in
/// a block, a line that is indented or opens with a closing bracket
/// continues the statement above.
pub fn statements(code: &str, block: bool) -> Vec<String> {
    if !block {
        return split_top(code, ";")
            .into_iter()
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
    }
    let mut parts: Vec<String> = Vec::new();
    for line in code.lines() {
        if line.trim().is_empty() || line.trim_start().starts_with("//") {
            continue;
        }
        match parts.last_mut() {
            Some(last) if line.starts_with([' ', '\t', ')', ']', '}']) => {
                last.push('\n');
                last.push_str(line);
            }
            _ => parts.push(line.to_string()),
        }
    }
    parts
}

/// A doc comment read as std/README.md writes one.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct DocText {
    /// The doc without its fact and `example:` lines. Fenced blocks stay.
    pub prose: String,
    pub facts: Facts,
    pub examples: Vec<Example>,
}

impl DocText {
    pub fn parse(doc: &str) -> DocText {
        let mut out = DocText::default();
        let mut prose = Vec::new();
        // (code start, indent, lines) of an open CP fence
        let mut fence: Option<(u32, usize, Vec<&str>)> = None;
        read(doc, &mut out.facts, &mut |at, line, kind| {
            if kind == Line::Example {
                return out.examples.push(one_line(line, at));
            }
            prose.push(line);
            match (kind, &mut fence) {
                (Line::Open, _) => {
                    fence = Some((at + 1, line.len() - line.trim_start().len(), Vec::new()))
                }
                (Line::Code, Some((_, _, code))) => code.push(line),
                (Line::Close, Some((start, indent, code))) => {
                    let code: Vec<&str> = code
                        .iter()
                        .map(|l| l.get(*indent..).unwrap_or(l.trim_start()))
                        .collect();
                    let code = code.join("\n") + "\n";
                    out.examples.push(Example {
                        code: Some(code.clone()),
                        text: code,
                        line: *start,
                        block: true,
                        stdin: None,
                        expect: None,
                    });
                    fence = None;
                }
                _ => {}
            }
        });
        out.prose = prose.join("\n").trim().to_string();
        out
    }
}

impl Facts {
    /// Only the facts of `doc`, as [`DocText::parse`] reads them.
    pub fn parse(doc: &str) -> Facts {
        let mut facts = Facts::default();
        read(doc, &mut facts, &mut |_, _, _| {});
        facts
    }
}

/// What a doc line that is not a fact is to [`DocText::parse`].
#[derive(Clone, Copy, PartialEq, Eq)]
enum Line {
    Prose,
    /// The text after `example:`.
    Example,
    /// The fence opening a CP code block, its code lines, its closing fence;
    /// all three are prose too.
    Open,
    Code,
    Close,
}

/// Reads `doc` once: its facts into `facts`, every other line to `each`.
fn read<'a>(doc: &'a str, facts: &mut Facts, each: &mut dyn FnMut(u32, &'a str, Line)) {
    // A label line is a fact only where a paragraph could start: first,
    // after a blank line, or after another fact or `example:`. A line
    // under a fact continues it.
    let (mut block_open, mut fact): (bool, Option<&str>) = (true, None);
    // Whether a fence is open, and whether it holds CP code.
    let mut fence: Option<bool> = None;
    for (at, line) in doc.lines().enumerate() {
        let at = at as u32;
        let marker = line.trim();
        let kind = if let Some(cp) = fence {
            match (marker == "```", cp) {
                (true, cp) => {
                    (fence, block_open) = (None, true);
                    if cp {
                        Line::Close
                    } else {
                        Line::Prose
                    }
                }
                (false, true) => Line::Code,
                (false, false) => Line::Prose,
            }
        } else if let Some(language) = marker.strip_prefix("```") {
            (fact, block_open) = (None, false);
            let cp = matches!(language.trim(), "" | "cp");
            fence = Some(cp);
            if cp {
                Line::Open
            } else {
                Line::Prose
            }
        } else if let Some(rest) = marker.strip_prefix("example:") {
            (fact, block_open) = (None, true);
            each(at, rest.trim(), Line::Example);
            continue;
        } else {
            if block_open {
                if let Some((label, text)) = fact_line(line) {
                    if let Some(slot) = facts.slot(label).filter(|slot| slot.is_none()) {
                        *slot = Some(text.to_string());
                        fact = Some(label);
                        continue;
                    }
                }
            }
            match fact
                .and_then(|label| facts.slot(label))
                .and_then(Option::as_mut)
            {
                Some(text) if !marker.is_empty() && fact_line(line).is_none() => {
                    text.push(' ');
                    text.push_str(marker);
                    continue;
                }
                _ => {
                    fact = None;
                    block_open = marker.is_empty();
                    Line::Prose
                }
            }
        };
        each(at, line, kind);
    }
}

/// `cost: O(n)` as `("cost", "O(n)")`: a word at the start of the line,
/// a colon and a space.
fn fact_line(line: &str) -> Option<(&str, &str)> {
    let (label, text) = line.split_once(": ")?;
    let text = text.trim();
    (label.bytes().all(|b| b.is_ascii_lowercase()) && !label.is_empty() && !text.is_empty())
        .then_some((label, text))
}

fn one_line(text: &str, line: u32) -> Example {
    let mut example = Example {
        code: None,
        text: text.to_string(),
        line,
        block: false,
        stdin: None,
        expect: None,
    };
    let Some((code, mut tail)) = text
        .strip_prefix('`')
        .and_then(|after| after.split_once('`'))
    else {
        return example;
    };
    let (mut stdin, mut expect) = (None, None);
    loop {
        tail = tail.trim();
        if tail.is_empty() {
            break;
        }
        if let Some(rest) = tail.strip_prefix("(stdin `") {
            let Some((given, rest)) = rest.split_once("`)") else {
                return example;
            };
            stdin = Some(given.to_string());
            tail = rest;
        } else if let Some(rest) = tail.strip_prefix("(empty stdin)") {
            stdin = Some(String::new());
            tail = rest;
        } else if let Some(rest) = tail
            .strip_prefix("(traps ")
            .or_else(|| tail.strip_prefix("(error "))
        {
            let Some((code, rest)) = rest.split_once(')') else {
                return example;
            };
            expect = Some(code.trim().to_string());
            tail = rest;
        } else {
            return example;
        }
    }
    example.code = Some(code.to_string());
    example.stdin = stdin;
    example.expect = expect;
    example
}

/// `code` split at each `sep` outside brackets, strings and char literals,
/// each part trimmed.
pub fn split_top<'a>(code: &'a str, sep: &str) -> Vec<&'a str> {
    let mut parts = Vec::new();
    let (mut depth, mut quote, mut escape, mut start) = (0usize, None, false, 0);
    let mut chars = code.char_indices();
    while let Some((at, c)) = chars.next() {
        if let Some(q) = quote {
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' => quote = Some(c),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            _ if depth == 0 && code[at..].starts_with(sep) => {
                parts.push(code[start..at].trim());
                start = at + sep.len();
                for _ in 1..sep.chars().count() {
                    chars.next();
                }
            }
            _ => {}
        }
    }
    parts.push(code[start..].trim());
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn facts_examples_and_prose_come_apart() {
        let doc = "Sums it.\n\ncost: O(n)\ntraps: never\ntrap: `x`\n\nexample: `let x = 1; x + 1 == 2`\nexample: `read() == [3]` (stdin `3`)\nexample: words, not code\n```cp\nlet a = 1\n    + 2\nassert(a == 3)\n```\nties: none here: really\n  and more\n\nA sentence ending in\norder: is prose";
        let d = DocText::parse(doc);
        assert_eq!(d.facts.cost.as_deref(), Some("O(n)"));
        assert_eq!(d.facts.traps.as_deref(), Some("never"));
        assert_eq!(d.facts.ties.as_deref(), Some("none here: really and more"));
        assert_eq!(d.facts.order, None);
        assert_eq!(Facts::parse(doc), d.facts);
        assert!(d.prose.contains("\ntrap: `x`\n"));
        assert_eq!(
            d.facts.stated().iter().map(|f| f.0).collect::<Vec<_>>(),
            ["cost", "traps", "ties"]
        );
        assert_eq!(d.prose, "Sums it.\n\ntrap: `x`\n\n```cp\nlet a = 1\n    + 2\nassert(a == 3)\n```\n\nA sentence ending in\norder: is prose");
        assert_eq!(d.examples.len(), 4);
        assert_eq!(
            d.examples[0].claim(),
            (
                vec!["let x = 1".to_string()],
                "x + 1".to_string(),
                Some("2".to_string())
            )
        );
        assert_eq!(d.examples[1].stdin.as_deref(), Some("3"));
        assert_eq!(d.examples[1].line, 7);
        assert_eq!(d.examples[2].code, None);
        let block = &d.examples[3];
        assert!(block.block);
        assert_eq!(block.line, 10);
        assert_eq!(block.statements(), ["let a = 1\n    + 2", "assert(a == 3)"]);
    }

    #[test]
    fn tags_and_top_level_splits() {
        let d = DocText::parse("example: `f(\";\", [a; b]) == 'x'` (empty stdin) (traps R0401)");
        let e = &d.examples[0];
        assert_eq!(e.stdin.as_deref(), Some(""));
        assert_eq!(e.expect.as_deref(), Some("R0401"));
        assert_eq!(e.claim().1, "f(\";\", [a; b])");
        assert_eq!(split_top("a == b == c", " == "), ["a", "b", "c"]);
    }

    #[test]
    fn doc_lines_above_a_declaration() {
        let d = DocText::parse("```text\nnot code (\n```\n```cp\nprint(1)\n```");
        assert_eq!(d.examples.len(), 1);
        assert_eq!(d.examples[0].code.as_deref(), Some("print(1)\n"));
        let src = "//// ruler\n/// One.\n/// Two.\n@cost(O(1))\nfn f() = 1\n";
        assert_eq!(doc_above(src, src.find("fn").unwrap()), "One.\nTwo.");
        assert_eq!(doc_line("  /// x"), Some("x"));
        assert_eq!(doc_line("//// x"), None);
    }
}

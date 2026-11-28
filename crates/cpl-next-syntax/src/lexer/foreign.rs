//! Lexing habits from other languages: Python's `//`, C++'s `int main() {`
//! and C's `i++`. Each is one error with the CP Lang form, then lexed as that.

use super::*;
use cpl_diag::habits;

impl<'a> Lexer<'a> {
    /// `n // 3)`: Python's floor division, which here starts a comment
    /// and hides the closer (or an inline `else:`). One error with the `/` fix, read as `/`.
    pub(super) fn floor_division(&mut self) -> bool {
        use TokenKind::{Float, Ident, Int, RBracket, RParen};
        let start = self.pos;
        let rest = self.src[start + 2..]
            .lines()
            .next()
            .unwrap_or("")
            .trim_start();
        let operand_before = matches!(
            self.last_kind(),
            Some(Ident | Int | Float | RParen | RBracket)
        );
        let operand_after =
            rest.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_' || c == '(');
        // Only a closer the rest of the line leaves unmatched: a trailing
        // comment like `// rows[v - 1] lists ..` balances its own brackets.
        let mut depth = 0u32;
        let closes_open = rest.chars().any(|c| match c {
            '(' | '[' => {
                depth += 1;
                false
            }
            ')' | ']' if depth == 0 => true,
            ')' | ']' => {
                depth -= 1;
                false
            }
            _ => false,
        });
        let hides_closer = !self.openers.is_empty() && closes_open;
        let line = &self.src[self.src[..start].rfind('\n').map_or(0, |i| i + 1)..start];
        let words = |s: &'a str| s.split(|c: char| !c.is_alphanumeric() && c != '_');
        let has = |kind| words(line).any(|w| TokenKind::keyword(w) == Some(kind));
        let open_if = has(TokenKind::If) && !has(TokenKind::Else);
        let hides_else = open_if && (rest.contains(" else:") || rest.contains(" else :"));
        if !operand_before || !operand_after || !(hides_closer || hides_else) {
            return false;
        }
        let span = Span::new(start as u32, start as u32 + 2);
        self.out.diagnostics.push(
            cpl_diag::msgs::lexer::FLOOR_DIV_IS_COMMENT
                .error(span, &[])
                .with_help(habits::FLOOR_DIVISION.say(&[]))
                .with_fix(habits::REPLACE_WITH.say(&[&"/"]), span, "/"),
        );
        self.pos += 2;
        self.push(TokenKind::Slash, start);
        true
    }

    /// C++'s `int main() {` with its body indented and its `}` alone at
    /// column 0: the body is the program, read one level out.
    pub(super) fn c_main_header(&mut self, begin: usize, end: usize, line: &str) -> bool {
        let head: String = line.trim_end_matches('{').split_whitespace().collect();
        let mains = [
            "intmain()",
            "intmain(void)",
            "signedmain()",
            "int32_tmain()",
        ];
        if self.c_main.is_some() || !line.ends_with('{') || !mains.contains(&head.as_str()) {
            return false;
        }
        let body = (end + 1).min(self.src.len());
        let mut starts = Vec::new();
        let mut at = body;
        let close = self.src[body..].split_inclusive('\n').find_map(|l| {
            if l.trim_end() == "}" {
                return Some(at);
            }
            starts.push(at);
            at += l.len();
            None
        });
        let Some(close) = close else {
            return false;
        };
        let src = self.src;
        let code = |b: &usize| !src[*b..].trim_start_matches(' ').starts_with(['\n', '\r']);
        let indent = |b: &usize| src[*b..].bytes().take_while(|&c| c == b' ').count();
        let lines: Vec<usize> = starts.into_iter().filter(code).collect();
        let width = lines.iter().map(indent).min().unwrap_or(0);
        if width == 0 || lines.iter().any(|b| src[b + width..].starts_with('\t')) {
            return false;
        }
        let close_end = close
            + self.src[close..]
                .find('\n')
                .unwrap_or(self.src.len() - close);
        let span = |from: usize, to: usize| Span::new(from as u32, to as u32);
        let mut edits = vec![(span(begin, body), String::new())];
        edits.extend(lines.iter().map(|&b| (span(b, b + width), String::new())));
        edits.push((
            span(close, (close_end + 1).min(self.src.len())),
            String::new(),
        ));
        let mut d = cpl_diag::msgs::lexer::NO_MAIN
            .error(span(begin, begin + line.len()), &[])
            .with_help(habits::C_MAIN.say(&[]));
        d.fixes.push(cpl_diag::Fix {
            title: habits::C_MAIN_2.say(&[]),
            edits,
        });
        self.out.diagnostics.push(d);
        self.c_main = Some((width as u32, close, close_end));
        self.pos = end;
        true
    }

    /// `i++` or `i--` ending a statement: C's increment, written `i += 1`
    /// here. The `++` is dropped so the line still reads as `i`, with no cascade.
    pub(super) fn increment(&mut self, start: usize) -> bool {
        let after = self.src[start + 2..].lines().next().unwrap_or("");
        let target = self
            .out
            .tokens
            .last()
            .filter(|t| t.span.end as usize == start);
        let place =
            target.is_some_and(|t| matches!(t.kind, TokenKind::Ident | TokenKind::RBracket));
        let line_start = self.src[..start].rfind('\n').map_or(0, |i| i + 1);
        let line = &self.src[line_start..start];
        let first = (self.out.tokens.iter().rev())
            .take_while(|t| t.span.start as usize >= line_start)
            .last();
        let c_for = first.is_some_and(|t| t.kind == TokenKind::For) && line.contains(';');
        let prefix = after.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_');
        let postfix = place && after.trim_start().starts_with(')');
        let (op, msg, ours) = match &self.src[start..start + 2] {
            "--" if c_for => return false,
            "--" => ("--", &cpl_diag::msgs::lexer::NO_DECREMENT, "-="),
            _ => ("++", &cpl_diag::msgs::lexer::NO_INCREMENT, "+="),
        };
        if c_for && (postfix || prefix && after.contains(')')) {
            // The parser reports the whole C-style `for` header once.
            self.pos += 2;
            return true;
        }
        if !place || !(after.trim().is_empty() || after.trim_start().starts_with("//")) {
            return false;
        }
        self.pos += 2;
        let span = Span::new(start as u32, self.pos as u32);
        self.out.diagnostics.push(
            msg.error(span, &[])
                .with_help(habits::WRITE.say(&[&format!("{ours} 1")]))
                .with_fix(
                    habits::INCREMENT.say(&[&op, &ours]),
                    span,
                    format!(" {ours} 1"),
                ),
        );
        true
    }
}

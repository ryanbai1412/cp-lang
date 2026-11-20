//! The indentation-sensitive lexer.
//!
//! Layout follows `plan/17-syntax.md`: blocks are indentation-based, spaces
//! only, a dedent must return to an enclosing level, and newlines inside
//! brackets don't end statements. The lexer turns that into `Newline`,
//! `Indent` and `Dedent` tokens so the parser never looks at whitespace.
//!
//! The lexer never fails. A character it can't read becomes an `Error`
//! token, and a bracket left open is closed for the parser (see
//! `close_unclosed`), so one typo stays one diagnostic.

use crate::token::{Token, TokenKind};
use cpl_diag::{codes, Diagnostic, Span};

/// The tokens of one file, and what was wrong with its layout.
#[derive(Debug, Clone, Default)]
pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
    /// Spans the parser and checker should stay quiet inside: the lexer
    /// already reported their root cause.
    pub poisoned: Vec<Span>,
    /// `//` comments, for the formatter and for doc comments.
    pub comments: Vec<Span>,
}

struct Opener {
    kind: TokenKind,
    span: Span,
    /// Indentation of the line the opener is on.
    line_indent: u32,
}

struct Lexer<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
    out: Lexed,
    indents: Vec<u32>,
    openers: Vec<Opener>,
    /// Indentation of the current logical line.
    line_indent: u32,
    /// Lines indented with tabs, reported once per file.
    tab_lines: Vec<Span>,
}

pub fn lex(src: &str) -> Lexed {
    let mut lx = Lexer {
        src,
        bytes: src.as_bytes(),
        pos: 0,
        out: Lexed::default(),
        indents: vec![0],
        openers: Vec::new(),
        line_indent: 0,
        tab_lines: Vec::new(),
    };
    lx.out.tokens.reserve(src.len() / 3);
    lx.run();
    lx.out
}

impl<'a> Lexer<'a> {
    fn peek(&self) -> u8 {
        self.bytes.get(self.pos).copied().unwrap_or(0)
    }

    fn peek_at(&self, n: usize) -> u8 {
        self.bytes.get(self.pos + n).copied().unwrap_or(0)
    }

    fn push(&mut self, kind: TokenKind, start: usize) {
        self.out.tokens.push(Token {
            kind,
            span: Span::new(start as u32, self.pos as u32),
        });
    }

    fn push_empty(&mut self, kind: TokenKind, at: usize) {
        self.out.tokens.push(Token {
            kind,
            span: Span::point(at as u32),
        });
    }

    fn last_kind(&self) -> Option<TokenKind> {
        self.out.tokens.last().map(|t| t.kind)
    }

    fn run(&mut self) {
        self.line_start();
        while self.pos < self.bytes.len() {
            let b = self.peek();
            match b {
                b' ' | b'\r' => self.pos += 1,
                b'\t' => self.pos += 1,
                b'\n' => {
                    let at = self.pos;
                    self.pos += 1;
                    if self.openers.is_empty() {
                        self.end_line(at);
                        self.line_start();
                    } else {
                        self.continuation_line();
                    }
                }
                b'/' if self.peek_at(1) == b'/' => self.comment(),
                _ => self.token(),
            }
        }
        let end = self.bytes.len();
        if !self.openers.is_empty() {
            self.close_unclosed(0, end, end);
        }
        self.end_line(end);
        while self.indents.len() > 1 {
            self.indents.pop();
            self.push_empty(TokenKind::Dedent, end);
        }
        self.push_empty(TokenKind::Eof, end);
        self.report_tabs();
    }

    fn comment(&mut self) {
        let start = self.pos;
        while self.pos < self.bytes.len() && self.peek() != b'\n' {
            self.pos += 1;
        }
        self.out.comments.push(Span::new(start as u32, self.pos as u32));
    }

    /// Ends the current logical line, if it had any tokens.
    fn end_line(&mut self, at: usize) {
        match self.last_kind() {
            None | Some(TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent) => {}
            _ => self.push_empty(TokenKind::Newline, at),
        }
    }

    /// Measures the indentation of the line starting at `self.pos`.
    /// Returns `None` for a blank or comment-only line.
    fn measure_indent(&mut self) -> Option<u32> {
        let start = self.pos;
        let mut width = 0u32;
        let mut tab = false;
        loop {
            match self.peek() {
                b' ' => width += 1,
                // A tab is reported, then counted as four spaces so the rest
                // of the file keeps its shape.
                b'\t' => {
                    width += 4;
                    tab = true;
                }
                _ => break,
            }
            self.pos += 1;
        }
        let blank = match self.peek() {
            b'\n' | b'\r' | 0 => true,
            b'/' => self.peek_at(1) == b'/',
            _ => false,
        };
        if blank && !(self.peek() == 0 && self.pos < self.bytes.len()) {
            return None;
        }
        if tab {
            self.tab_lines.push(Span::new(start as u32, self.pos as u32));
        }
        Some(width)
    }

    /// Handles the start of a line outside brackets: emits `Indent` or
    /// `Dedent` tokens for the change in indentation.
    fn line_start(&mut self) {
        let start = self.pos;
        let Some(width) = self.measure_indent() else { return };
        self.line_indent = width;
        let top = *self.indents.last().unwrap();
        if width > top {
            self.indents.push(width);
            self.push_empty(TokenKind::Indent, self.pos);
        } else if width < top {
            while *self.indents.last().unwrap() > width {
                self.indents.pop();
                self.push_empty(TokenKind::Dedent, self.pos);
            }
            let enclosing = *self.indents.last().unwrap();
            if enclosing != width {
                // The line sits between two levels. Treat it as part of the
                // enclosing block it is indented past.
                let levels: Vec<String> = self.indents.iter().map(|w| w.to_string()).collect();
                let span = Span::new(start as u32, self.pos as u32);
                let fix_to = " ".repeat(enclosing as usize);
                self.out.diagnostics.push(
                    Diagnostic::error(
                        codes::BAD_DEDENT,
                        span,
                        format!(
                            "this line is indented {width} spaces, which matches no enclosing block ({})",
                            levels.join(" or ")
                        ),
                    )
                    .with_help(format!("indent it {enclosing} spaces to continue the enclosing block"))
                    .with_fix(format!("indent {enclosing} spaces"), span, fix_to),
                );
                self.line_indent = enclosing;
            }
        }
    }

    /// Handles the start of a line inside brackets. Newlines there don't end
    /// statements, unless the line shows the bracket was never closed.
    fn continuation_line(&mut self) {
        let line_start = self.pos;
        let Some(width) = self.measure_indent() else { return };
        let Some(depth) = self.unclosed_depth(width) else { return };
        // Close at the end of the last line that had tokens.
        let close_at = self.out.tokens.last().map_or(line_start, |t| t.span.end as usize);
        self.close_unclosed(depth, close_at, line_start);
        self.end_line(close_at);
        self.pos = line_start;
        self.line_start();
    }

    /// If a line indented `width` spaces can't be a continuation of the open
    /// brackets, returns how many of them stay open.
    ///
    /// A continuation line is indented past the line that opened the
    /// bracket, or starts with a closer. A line that is not, and that starts
    /// with a name or a statement keyword, begins a new statement: the
    /// bracket was left open. (A line starting with a literal or an operator
    /// is left alone, so `[\n1, 2\n]` still reads as one list.)
    fn unclosed_depth(&self, width: u32) -> Option<usize> {
        let keep = self.openers.iter().take_while(|o| o.line_indent < width).count();
        if keep == self.openers.len() {
            return None;
        }
        let rest = &self.bytes[self.pos..];
        let word_len = rest
            .iter()
            .take_while(|b| b.is_ascii_alphanumeric() || **b == b'_')
            .count();
        let first = rest.first().copied().unwrap_or(0);
        let starts_statement = if first == b'@' {
            true
        } else if first.is_ascii_alphabetic() || first == b'_' {
            let word = &self.src[self.pos..self.pos + word_len];
            match TokenKind::keyword(word) {
                Some(kind) => kind.starts_statement(),
                None => true,
            }
        } else {
            false
        };
        // A raw string `r"..."` starts with a letter but is a literal.
        let raw_string = first == b'r' && word_len == 1 && rest.get(1) == Some(&b'"');
        (starts_statement && !raw_string).then_some(keep)
    }

    /// Reports every opener past `keep` as unclosed and gives the parser the
    /// closers it is missing, innermost first.
    fn close_unclosed(&mut self, keep: usize, close_at: usize, next_line: usize) {
        let outermost = self.openers[keep].span;
        while self.openers.len() > keep {
            let opener = self.openers.pop().unwrap();
            let (open, close, closer_kind) = delimiter_text(opener.kind);
            // Only the outermost is reported: inner ones are the same typo
            // far more often than a second one.
            if self.openers.len() == keep {
                let mut diag = Diagnostic::error(
                    codes::UNCLOSED_DELIMITER,
                    opener.span,
                    format!("this `{open}` is never closed"),
                )
                .with_help(format!("add `{close}` at the end of the expression"))
                .with_fix(format!("insert `{close}`"), Span::point(close_at as u32), close);
                if next_line > close_at {
                    diag = diag.with_label(
                        Span::point(next_line as u32),
                        "a new statement starts here",
                    );
                }
                self.out.diagnostics.push(diag);
            }
            self.push_empty(closer_kind, close_at);
        }
        self.out
            .poisoned
            .push(Span::new(outermost.end, close_at.max(outermost.end as usize) as u32));
    }

    fn token(&mut self) {
        let start = self.pos;
        let b = self.peek();
        if b.is_ascii_alphabetic() || b == b'_' {
            if b == b'r' && self.peek_at(1) == b'"' {
                self.pos += 1;
                self.string(start, true);
                return;
            }
            while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                self.pos += 1;
            }
            let kind = TokenKind::keyword(&self.src[start..self.pos]).unwrap_or(TokenKind::Ident);
            self.push(kind, start);
            return;
        }
        if b.is_ascii_digit() {
            self.number(start);
            return;
        }
        use TokenKind::*;
        let (kind, len) = match (b, self.peek_at(1), self.peek_at(2)) {
            (b'"', _, _) => {
                self.string(start, false);
                return;
            }
            (b'\'', _, _) => {
                self.char_or_label(start);
                return;
            }
            (b'(' | b'[' | b'{', _, _) => {
                let kind = match b {
                    b'(' => LParen,
                    b'[' => LBracket,
                    _ => LBrace,
                };
                self.pos += 1;
                self.openers.push(Opener {
                    kind,
                    span: Span::new(start as u32, self.pos as u32),
                    line_indent: self.line_indent,
                });
                self.push(kind, start);
                return;
            }
            (b')' | b']' | b'}', _, _) => {
                self.closer(start);
                return;
            }
            (b'.', b'.', b'=') => (DotDotEq, 3),
            (b'.', b'.', _) => (DotDot, 2),
            (b'.', _, _) => (Dot, 1),
            (b',', _, _) => (Comma, 1),
            (b':', b':', _) => (ColonColon, 2),
            (b':', _, _) => (Colon, 1),
            (b';', _, _) => (Semi, 1),
            (b'@', _, _) => (At, 1),
            (b'?', b'?', _) => (QuestionQuestion, 2),
            (b'?', _, _) => (Question, 1),
            (b'-', b'>', _) => (Arrow, 2),
            (b'=', b'>', _) => (FatArrow, 2),
            (b'=', b'=', _) => (EqEq, 2),
            (b'=', _, _) => (Eq, 1),
            (b'!', b'=', _) => (NotEq, 2),
            (b'!', _, _) => (Bang, 1),
            (b'*', b'*', b'=') => (OpEq, 3),
            (b'<', b'<', b'=') => (OpEq, 3),
            (b'>', b'>', b'=') => (OpEq, 3),
            (b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'&' | b'|', b'=', _) => (OpEq, 2),
            (b'<', b'=', b'>') => (Spaceship, 3),
            (b'<', b'=', _) => (LtEq, 2),
            (b'>', b'=', _) => (GtEq, 2),
            (b'<', b'<', _) => (Shl, 2),
            (b'>', b'>', _) => (Shr, 2),
            (b'<', _, _) => (Lt, 1),
            (b'>', _, _) => (Gt, 1),
            (b'+', _, _) => (Plus, 1),
            (b'-', _, _) => (Minus, 1),
            (b'*', b'*', _) => (StarStar, 2),
            (b'*', _, _) => (Star, 1),
            (b'/', _, _) => (Slash, 1),
            (b'%', _, _) => (Percent, 1),
            (b'^', _, _) => (Caret, 1),
            (b'&', _, _) => (Amp, 1),
            (b'|', _, _) => (Pipe, 1),
            (b'~', _, _) => (Tilde, 1),
            _ => {
                let ch = self.src[start..].chars().next().unwrap();
                self.pos += ch.len_utf8();
                let span = Span::new(start as u32, self.pos as u32);
                let mut diag = Diagnostic::error(
                    codes::UNKNOWN_CHAR,
                    span,
                    format!("`{ch}` isn't used in cp"),
                );
                diag.help = match ch {
                    '#' => Some("comments start with `//`".into()),
                    _ if !ch.is_ascii() => Some("names are ASCII; other characters go in strings and comments".into()),
                    _ => Option::None,
                };
                self.out.diagnostics.push(diag);
                self.out.poisoned.push(span);
                self.push(Error, start);
                return;
            }
        };
        self.pos += len;
        self.push(kind, start);
    }

    fn closer(&mut self, start: usize) {
        use TokenKind::*;
        let kind = match self.peek() {
            b')' => RParen,
            b']' => RBracket,
            _ => RBrace,
        };
        let opener_kind = match kind {
            RParen => LParen,
            RBracket => LBracket,
            _ => LBrace,
        };
        // `(a]`: if an enclosing opener matches, the inner ones are unclosed.
        // If none does, the closer is stray and the parser reports it.
        if let Some(at) = self.openers.iter().rposition(|o| o.kind == opener_kind) {
            if at + 1 < self.openers.len() {
                self.close_unclosed(at + 1, start, start);
            }
            self.openers.pop();
        }
        self.pos += 1;
        self.push(kind, start);
    }

    fn number(&mut self, start: usize) {
        let radix_prefix = self.peek() == b'0' && matches!(self.peek_at(1), b'x' | b'X' | b'b' | b'B');
        let mut float = false;
        if radix_prefix {
            self.pos += 2;
            while self.peek().is_ascii_hexdigit() || self.peek() == b'_' {
                self.pos += 1;
            }
        } else {
            self.digits();
            // `0..n` is a range, and `x.0` or `1.max(2)` are not fractions.
            if self.peek() == b'.' && self.peek_at(1).is_ascii_digit() {
                float = true;
                self.pos += 1;
                self.digits();
            }
            if matches!(self.peek(), b'e' | b'E') {
                let sign = matches!(self.peek_at(1), b'+' | b'-') as usize;
                if self.peek_at(1 + sign).is_ascii_digit() {
                    self.pos += 1 + sign;
                    self.digits();
                }
            }
        }
        // `12abc` is one bad token, not a number followed by a name.
        let digits_end = self.pos;
        while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
            self.pos += 1;
        }
        let text = &self.src[start..self.pos];
        if self.pos > digits_end || (radix_prefix && digits_end == start + 2) {
            self.out.diagnostics.push(
                Diagnostic::error(
                    codes::BAD_NUMBER,
                    Span::new(start as u32, self.pos as u32),
                    format!("`{text}` isn't a number"),
                )
                .with_help("numbers look like `42`, `1_000_000`, `1e9`, `0xff`, `0b1010` or `0.5`"),
            );
            self.push(TokenKind::Int, start);
            return;
        }
        let kind = if float || crate::literal::int_value(text).is_none() {
            TokenKind::Float
        } else {
            TokenKind::Int
        };
        self.push(kind, start);
    }

    fn digits(&mut self) {
        while self.peek().is_ascii_digit() || self.peek() == b'_' {
            self.pos += 1;
        }
    }

    /// Lexes a string literal starting at its opening quote (or at the `r`
    /// of a raw string, already consumed). Interpolations are left in the
    /// token; `literal::string_parts` splits them for the parser.
    fn string(&mut self, start: usize, raw: bool) {
        self.pos += 1;
        let closed = self.string_body(raw);
        if !closed {
            let span = Span::new(start as u32, self.pos as u32);
            self.out.diagnostics.push(
                Diagnostic::error(codes::UNTERMINATED_STRING, span, "this string is never closed")
                    .with_help("add `\"` at the end of the string")
                    .with_fix("insert `\"`", Span::point(self.pos as u32), "\""),
            );
            self.out.poisoned.push(span);
        }
        self.push(if raw { TokenKind::RawStr } else { TokenKind::Str }, start);
    }

    /// Scans to the closing quote. Returns false if the line ends first.
    fn string_body(&mut self, raw: bool) -> bool {
        let mut depth = 0u32;
        loop {
            match self.peek() {
                b'\n' | 0 => return false,
                b'\\' if !raw => {
                    self.pos += 1;
                    if !matches!(self.peek(), b'\n' | 0) {
                        self.pos += 1;
                    }
                }
                b'"' if depth == 0 => {
                    self.pos += 1;
                    return true;
                }
                // A string inside an interpolation: `"{names["x"]}"`.
                b'"' => {
                    self.pos += 1;
                    if !self.string_body(false) {
                        return false;
                    }
                }
                b'{' if !raw => {
                    if depth == 0 && self.peek_at(1) == b'{' {
                        self.pos += 2;
                    } else {
                        depth += 1;
                        self.pos += 1;
                    }
                }
                b'}' if !raw => {
                    depth = depth.saturating_sub(1);
                    self.pos += 1;
                }
                _ => self.pos += 1,
            }
        }
    }

    fn char_or_label(&mut self, start: usize) {
        self.pos += 1;
        let rest = &self.src[self.pos..];
        let mut chars = rest.chars();
        let first = chars.next();
        let char_len = match first {
            Some('\\') => chars.next().map(|c| 1 + c.len_utf8()),
            Some(c) if c != '\n' && c != '\'' => Some(c.len_utf8()),
            _ => None,
        };
        if let Some(len) = char_len {
            if rest.as_bytes().get(len) == Some(&b'\'') {
                self.pos += len + 1;
                self.push(TokenKind::Char, start);
                return;
            }
        }
        if first.is_some_and(|c| c.is_ascii_alphabetic() || c == '_') {
            while self.peek().is_ascii_alphanumeric() || self.peek() == b'_' {
                self.pos += 1;
            }
            self.push(TokenKind::Label, start);
            return;
        }
        let span = Span::new(start as u32, self.pos as u32);
        self.out.poisoned.push(span);
        self.out.diagnostics.push(
            Diagnostic::error(
                codes::BAD_CHAR_LITERAL,
                span,
                "a character is written `'a'`, with exactly one character between the quotes",
            )
            .with_help("use double quotes for a string"),
        );
        self.push(TokenKind::Error, start);
    }

    fn report_tabs(&mut self) {
        let Some(first) = self.tab_lines.first().copied() else { return };
        let mut diag = Diagnostic::error(
            codes::TAB_INDENT,
            first,
            "this line is indented with a tab; cp indents with spaces only",
        )
        .with_help("replace each tab with 4 spaces");
        if self.tab_lines.len() > 1 {
            diag = diag.with_note(format!(
                "{} more lines in this file are indented with tabs",
                self.tab_lines.len() - 1
            ));
        }
        let edits = self
            .tab_lines
            .iter()
            .map(|span| (*span, span.text(self.src).replace('\t', "    ")))
            .collect();
        diag.fixes.push(cpl_diag::Fix {
            title: "replace tabs with spaces".into(),
            edits,
        });
        self.out.diagnostics.push(diag);
    }
}

fn delimiter_text(opener: TokenKind) -> (&'static str, &'static str, TokenKind) {
    match opener {
        TokenKind::LParen => ("(", ")", TokenKind::RParen),
        TokenKind::LBracket => ("[", "]", TokenKind::RBracket),
        _ => ("{", "}", TokenKind::RBrace),
    }
}

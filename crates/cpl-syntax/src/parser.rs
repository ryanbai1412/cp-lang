//! The error-recovering parser.
//!
//! The parser always returns a full tree and reports at most one diagnostic
//! per statement: after the first error in a statement it is *recovering*,
//! stays silent, and resynchronizes at the end of the line. The header it
//! could not read is poisoned so later phases don't report on it either.
//! Blocks nested under a broken header are still parsed normally, so a
//! missing `:` on a `for` line doesn't hide the errors, or the hovers, in
//! its body.

mod expr;
mod foreign;
mod item;
mod pattern;
mod stmt;
mod types;

use crate::ast::*;
use crate::token::{Token, TokenKind};
use cpl_diag::{codes, Code, Diagnostic, Sink, Span};
use TokenKind as T;

/// One parsed top-level item: usually one statement, with spans relative to
/// the item's start.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedItem {
    pub stmts: Vec<Stmt>,
    pub diagnostics: Vec<Diagnostic>,
    /// Spans later phases must not report inside.
    pub poisoned: Vec<Span>,
}

/// Parses the tokens of one top-level item.
///
/// `tokens` carry spans into `src`; `base` is where the item starts, and is
/// subtracted from every span in the result. `lexer_poison` holds the spans
/// the lexer already reported on, relative to the item.
pub fn parse_item(src: &str, tokens: &[Token], base: u32, lexer_poison: Vec<Span>) -> ParsedItem {
    let mut p = Parser::new(
        src,
        tokens,
        -(base as i64),
        Sink::with_poisoned(lexer_poison),
    );
    let mut stmts = Vec::new();
    while !p.at(T::Eof) {
        let before = p.pos;
        p.stmt_into(&mut stmts);
        if p.pos == before {
            p.bump();
        }
    }
    let poisoned = p.sink.poisoned().to_vec();
    ParsedItem {
        stmts,
        diagnostics: p.sink.finish(),
        poisoned,
    }
}

pub(crate) struct Parser<'a> {
    /// The `var` parameters of the fn whose signature is being read.
    var_params: Vec<Name>,
    /// `fn f((a, b): P)`: each pattern and the name its parameter gets.
    param_patterns: Vec<(Pattern, Name)>,
    src: &'a str,
    tokens: &'a [Token],
    pos: usize,
    /// Added to a token's span to make it relative to the item.
    delta: i64,
    sink: Sink,
    /// An error was reported in the current statement; stay quiet until the
    /// statement ends.
    recovering: bool,
    /// Depth of speculative parsing. Errors are counted, not reported.
    speculating: u32,
    speculation_failed: bool,
    /// Planned syntax read while speculating, reported if the speculation holds.
    speculative_planned: Vec<(Span, &'static crate::planned::Planned)>,
    /// Inside `{ ... }`, where `;` separates statements.
    brace_depth: u32,
    /// How many blocks deep the statement being read is: 0 at the top
    /// level of the file.
    block_depth: u32,
    /// Inside a trait body, where fns and consts may lack bodies.
    in_trait: bool,
    /// Inside an `impl` body, where operator fns desugar directly to methods.
    in_impl: bool,
    /// Inside a `@grader` statement, where a fn may have no body: the
    /// judge supplies it.
    in_grader: bool,
    /// The next comparison starts a statement, where `x min= y` is an
    /// assignment and not a `Bool`.
    stmt_head: bool,
    /// The first half of a `>>` was used to close a type argument list.
    half_shr: bool,
    /// Statements the one being parsed stands for besides itself: the
    /// later `let`s of `let a = x and let b = y else: ...`.
    extra_stmts: Vec<crate::ast::Stmt>,
    /// Statements that run before the one being read: the `m[k] ??= init`
    /// of `(m[k] ??= init).push(x)`.
    lead_stmts: Vec<crate::ast::Stmt>,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str, tokens: &'a [Token], delta: i64, sink: Sink) -> Parser<'a> {
        Parser {
            src,
            tokens,
            pos: 0,
            delta,
            sink,
            recovering: false,
            var_params: Vec::new(),
            param_patterns: Vec::new(),
            speculating: 0,
            speculation_failed: false,
            speculative_planned: Vec::new(),
            stmt_head: false,
            brace_depth: 0,
            block_depth: 0,
            in_trait: false,
            in_impl: false,
            in_grader: false,
            half_shr: false,
            extra_stmts: Vec::new(),
            lead_stmts: Vec::new(),
        }
    }

    // --- Tokens ---------------------------------------------------------

    fn peek(&self) -> TokenKind {
        self.peek_n(0)
    }

    fn peek_n(&self, n: usize) -> TokenKind {
        self.tokens.get(self.pos + n).map_or(T::Eof, |t| t.kind)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn rel(&self, span: Span) -> Span {
        Span::new(
            (span.start as i64 + self.delta) as u32,
            (span.end as i64 + self.delta) as u32,
        )
    }

    /// The span of the current token, relative to the item.
    fn span(&self) -> Span {
        match self.tokens.get(self.pos) {
            Some(t) => self.rel(t.span),
            None => Span::point(self.prev_end()),
        }
    }

    /// The span of the token `n` ahead, relative to the item.
    fn span_n(&self, n: usize) -> Span {
        match self.tokens.get(self.pos + n) {
            Some(t) => self.rel(t.span),
            None => Span::point(self.prev_end()),
        }
    }

    /// Past `..` or `..=`, and past a `<` written right after `..`, which
    /// is reported with the fix to drop it.
    fn range_op(&mut self) {
        let inclusive = self.at(T::DotDotEq);
        self.bump();
        if !inclusive && self.at(T::Lt) && self.span().start == self.prev_end() {
            let span = self.span();
            self.error(
                Diagnostic::error(codes::UNEXPECTED_TOKEN, span, "`..<` is `..` in cp")
                    .with_help("`lo..hi` already leaves out `hi`; `lo..=hi` keeps it")
                    .with_fix("write `..`", span, ""),
            );
            self.bump();
        }
    }

    /// `half` twice with nothing between: `&&` or `||`.
    fn doubled(&self, half: TokenKind) -> bool {
        self.at(half) && self.peek_n(1) == half && self.span().end == self.span_n(1).start
    }

    fn start(&self) -> u32 {
        self.span().start
    }

    /// Where the previous token ended; after the first half of a `>>` that
    /// closes two type-argument lists, just past that half.
    fn prev_end(&self) -> u32 {
        if self.half_shr {
            if let Some(t) = self.tokens.get(self.pos) {
                return self.rel(t.span).start + 1;
            }
        }
        match self.pos.checked_sub(1).and_then(|i| self.tokens.get(i)) {
            Some(t) => self.rel(t.span).end,
            None => self.tokens.first().map_or(0, |t| self.rel(t.span).start),
        }
    }

    fn prev_kind(&self) -> Option<TokenKind> {
        self.pos
            .checked_sub(1)
            .and_then(|i| self.tokens.get(i))
            .map(|t| t.kind)
    }

    fn span_from(&self, start: u32) -> Span {
        Span::new(start, self.prev_end().max(start))
    }

    fn text(&self) -> &'a str {
        match self.tokens.get(self.pos) {
            Some(t) => t.span.text(self.src),
            None => "",
        }
    }

    fn bump(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
        // A character the lexer could not read is already reported. Reading
        // on as if it were not there keeps the rest of the statement, and
        // the names it declares; the statement is still poisoned, so
        // nothing else is reported in it. At the start of a line it is left
        // for `stmt` to skip with the line.
        let line_start = matches!(
            self.tokens.get(self.pos.wrapping_sub(1)).map(|t| t.kind),
            Some(T::Newline | T::Indent | T::Dedent)
        );
        while !line_start
            && self
                .tokens
                .get(self.pos)
                .is_some_and(|t| t.kind == T::Error)
        {
            self.pos += 1;
            if self.speculating == 0 {
                self.recovering = true;
            }
        }
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        let hit = self.at(kind);
        if hit {
            self.bump();
        }
        hit
    }

    /// True at a token that ends the current line or block.
    fn at_line_end(&self) -> bool {
        matches!(self.peek(), T::Newline | T::Dedent | T::Eof | T::Indent)
    }

    /// True at `name:`, where the name may be spelled like a keyword
    /// (`by: cmp`, `in: xs`).
    fn at_label(&self) -> bool {
        self.peek_n(1) == T::Colon
            && (self.at(T::Ident)
                || (TokenKind::keyword(self.text()).is_some() && !self.at(T::Else)))
    }

    fn eat_ident(&mut self, text: &str) -> bool {
        let hit = self.at_ident(text);
        if hit {
            self.bump();
        }
        hit
    }

    fn at_ident(&self, text: &str) -> bool {
        self.at(T::Ident) && self.text() == text
    }

    fn name(&mut self) -> Name {
        let name = Name {
            text: self.text().to_string(),
            span: self.span(),
        };
        self.bump();
        name
    }

    /// A name, or an empty one (with an error) if none is here.
    fn expect_name(&mut self, what: &str) -> Name {
        if self.at(T::Ident) {
            return self.name();
        }
        self.error_expected(codes::EXPECTED_NAME, what, None);
        Name {
            text: String::new(),
            span: Span::point(self.prev_end()),
        }
    }

    // --- Errors ---------------------------------------------------------

    fn error(&mut self, diag: Diagnostic) {
        if self.speculating > 0 {
            self.speculation_failed = true;
        } else if !self.recovering {
            self.recovering = true;
            self.sink.push(diag);
        }
    }

    /// Reports an error the parser has fully recovered from (the tree reads
    /// as the fix would), so later errors in the statement still show.
    fn recovered(&mut self, diag: Diagnostic) {
        if self.speculating > 0 {
            self.speculation_failed = true;
        } else {
            self.sink.push(diag);
        }
    }

    /// Where an "expected X" error points: at the offending token if it is
    /// on this line, otherwise just past the last token of the line.
    fn error_span(&self) -> Span {
        if self.at_line_end() {
            Span::point(self.prev_end())
        } else {
            self.span()
        }
    }

    /// Reports "expected `what`, found <current token>".
    fn error_expected(&mut self, code: Code, what: &str, fix: Option<&str>) {
        let span = self.error_span();
        let found = self.peek().describe();
        let mut diag = Diagnostic::error(code, span, format!("expected {what}, found {found}"));
        if let Some(text) = fix {
            diag = diag.with_fix(format!("insert `{text}`"), Span::point(span.start), text);
        }
        self.error(diag);
    }

    /// Expects `kind`, described as `what` ("`)` to close the call").
    fn expect(&mut self, kind: TokenKind, what: &str) -> bool {
        if self.eat(kind) {
            return true;
        }
        let fix = match kind {
            T::RParen => Some(")"),
            T::RBracket => Some("]"),
            T::RBrace => Some("}"),
            T::Colon => Some(":"),
            T::Comma => Some(","),
            _ => None,
        };
        self.error_expected(codes::EXPECTED_TOKEN, what, fix);
        false
    }

    /// Runs `f` without reporting errors. Returns `None`, with the position
    /// restored, if `f` hit one.
    fn speculate<R>(&mut self, f: impl FnOnce(&mut Self) -> R) -> Option<R> {
        let (pos, failed, half) = (self.pos, self.speculation_failed, self.half_shr);
        let planned = self.speculative_planned.len();
        self.speculating += 1;
        self.speculation_failed = false;
        let result = f(self);
        let ok = !self.speculation_failed;
        self.speculating -= 1;
        self.speculation_failed = failed;
        if ok {
            if self.speculating == 0 {
                for (span, p) in std::mem::take(&mut self.speculative_planned) {
                    self.planned(span, p);
                }
            }
            Some(result)
        } else {
            self.speculative_planned.truncate(planned);
            self.pos = pos;
            self.half_shr = half;
            None
        }
    }

    /// Skips the rest of the current line, including any block indented
    /// under it, and stops before the token that ends the statement.
    fn skip_line(&mut self) {
        let mut indent = 0u32;
        let mut brackets = 0u32;
        loop {
            match self.peek() {
                T::Eof => break,
                T::Newline if indent == 0 && self.peek_n(1) != T::Indent => break,
                T::Dedent if indent == 0 => break,
                T::Dedent if indent == 1 => {
                    self.bump();
                    break;
                }
                T::Semi | T::RBrace if self.brace_depth > 0 && brackets == 0 && indent == 0 => {
                    break
                }
                T::Indent => indent += 1,
                T::Dedent => indent -= 1,
                T::LParen | T::LBracket | T::LBrace => brackets += 1,
                T::RParen | T::RBracket | T::RBrace => brackets = brackets.saturating_sub(1),
                _ => {}
            }
            self.bump();
        }
    }

    /// Skips to the next `,` or `close` at this bracket depth, or to the end
    /// of the line.
    fn skip_to(&mut self, close: TokenKind) {
        let mut brackets = 0u32;
        loop {
            let kind = self.peek();
            if self.at_line_end() {
                break;
            }
            if brackets == 0 && (kind == close || kind == T::Comma) {
                break;
            }
            match kind {
                T::LParen | T::LBracket | T::LBrace => brackets += 1,
                T::RParen | T::RBracket | T::RBrace => {
                    if brackets == 0 {
                        break;
                    }
                    brackets -= 1;
                }
                _ => {}
            }
            self.bump();
        }
    }

    /// Parses `item (, item)* [,] close`. The opener is already consumed.
    /// `what` names the list for the error: "the argument list".
    fn comma_list<R>(
        &mut self,
        close: TokenKind,
        what: &str,
        mut item: impl FnMut(&mut Self) -> R,
    ) -> Vec<R> {
        let mut out = Vec::new();
        loop {
            if self.at(close) || self.at_line_end() {
                break;
            }
            let before = self.pos;
            out.push(item(self));
            if self.eat(T::Comma) {
                continue;
            }
            if self.at(close) {
                break;
            }
            let expected = format!("`,` or {} in {what}", close.describe());
            self.error_expected(codes::EXPECTED_TOKEN, &expected, None);
            self.skip_to(close);
            if self.pos == before && !self.at(close) && !self.at_line_end() {
                self.bump();
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        let expected = format!("{} to close {what}", close.describe());
        self.expect(close, &expected);
        out
    }

    /// Reports `span` as planned syntax. The parser has read it, so it is
    /// poisoned rather than recovered from: what it declares stays as
    /// written.
    fn planned(&mut self, span: Span, planned: &'static crate::planned::Planned) {
        if self.speculating > 0 {
            self.speculative_planned.push((span, planned));
        } else {
            let message = planned.message();
            self.sink.push(
                Diagnostic::error(codes::NOT_IMPLEMENTED, span, message).with_help(planned.now),
            );
            self.sink.poison(span);
        }
    }

    /// Reports `span` as a form the language leaves out on purpose. Like
    /// planned syntax, it is read and poisoned rather than recovered from.
    fn not_in_language(&mut self, span: Span, message: &str, help: &str) {
        if self.speculating > 0 {
            self.speculation_failed = true;
        } else {
            self.sink
                .push(Diagnostic::error(codes::UNSUPPORTED, span, message).with_help(help));
            self.sink.poison(span);
        }
    }

    /// Where a missing block is reported: the next line, which was meant to
    /// be indented under the header, else the end of the header itself.
    fn block_expected_at(&self, header_end: u32) -> Span {
        let abs = (header_end as i64 - self.delta) as usize;
        let rest = self.src.get(abs..).unwrap_or("");
        let Some(nl) = rest.find('\n') else {
            return Span::point(header_end);
        };
        let after = &rest[nl + 1..];
        let lead = after.len() - after.trim_start().len();
        let line = after[lead..].split('\n').next().unwrap_or("").trim_end();
        if line.is_empty() || line.starts_with("//") {
            return Span::point(header_end);
        }
        let start = header_end + (nl + 1 + lead) as u32;
        Span::new(start, start + line.len() as u32)
    }

    /// Skips the indented block that starts here, through its dedent.
    fn skip_indented(&mut self) {
        let mut depth = 0u32;
        loop {
            match self.peek() {
                T::Indent => depth += 1,
                T::Dedent => depth = depth.saturating_sub(1),
                T::Eof => return,
                _ => {}
            }
            self.bump();
            if depth == 0 {
                return;
            }
        }
    }

    /// `let x =` with the value on the indented lines below: read past it
    /// as planned syntax, leaving an error value.
    fn continued_value(&mut self) -> Option<Expr> {
        if !(self.at(T::Newline) && self.peek_n(1) == T::Indent)
            || matches!(self.peek_n(2), T::If | T::Match)
        {
            return None;
        }
        let start = self.start();
        self.bump();
        self.skip_indented();
        let span = self.span_from(start);
        self.planned(span, &crate::planned::VALUE_BLOCK);
        Some(Expr {
            kind: ExprKind::Error,
            span,
        })
    }

    /// Ends error recovery for a statement that began at `start`: later
    /// phases stay quiet about the part the parser could not read.
    fn end_recovery(&mut self, start: u32) {
        if self.recovering {
            self.recovering = false;
            let span = self.span_from(start);
            self.sink.poison(span);
        }
    }

    /// `fn f() -> T =` with statements on the indented lines below: R01.11.1
    /// asks for `:`, and the body is read as the block it is.
    fn eq_block_body(&mut self, name: &Name) -> Option<Block> {
        if !(self.at(T::Eq) && self.peek_n(1) == T::Newline && self.peek_n(2) == T::Indent)
            || !matches!(
                self.peek_n(3),
                T::Let | T::Var | T::For | T::While | T::Return
            )
        {
            return None;
        }
        let (eq, from) = (self.span(), self.prev_end());
        self.error(
            Diagnostic::error(
                codes::REMOVED_SYNTAX,
                eq,
                "a body on the lines below starts with `:`, not `=`",
            )
            .with_help("`=` takes a body on the same line: `fn f() -> Int = a + 1`")
            .with_fix("replace `=` with `:`", Span::new(from, eq.end), ":"),
        );
        self.bump();
        Some(self.block(&format!("the signature of `{}`", name.text)))
    }

    /// `{k: v}`, a map written the other languages' way: `[k: v]` in cp.
    fn brace_map(&mut self, span: Span, open: Span, close: Span) {
        let mut diag = Diagnostic::error(
            codes::REMOVED_SYNTAX,
            span,
            "a map literal is written in brackets, `[k: v]`, not braces",
        )
        .with_help("braces make a block; brackets make lists and maps");
        diag.fixes.push(cpl_diag::Fix {
            title: "write it in brackets".into(),
            edits: vec![(open, "[".into()), (close, "]".into())],
        });
        self.error(diag);
    }

    /// `{k: v}` (`Some(None)`) or `{x for x in xs}` on one line: a map or
    /// set, not a block.
    fn brace_literal(&self) -> Option<Option<&'static crate::planned::Planned>> {
        let (mut depth, mut colon, mut comp, mut keyword) = (0u32, false, false, false);
        for (i, t) in self.tokens.iter().enumerate().skip(self.pos) {
            match t.kind {
                T::LParen | T::LBracket | T::LBrace => depth += 1,
                T::RParen | T::RBracket | T::RBrace => {
                    depth -= 1;
                    if depth == 0 {
                        return if comp && !colon {
                            Some(Some(&crate::planned::BRACE_SET))
                        } else if colon {
                            Some(None)
                        } else {
                            None
                        };
                    }
                }
                T::Newline | T::Indent | T::Dedent | T::Eof => return None,
                T::Colon if depth == 1 && !keyword && i > self.pos + 1 => colon = true,
                T::If | T::Elif | T::Else | T::While | T::Let | T::Var | T::Match | T::Return
                    if depth == 1 =>
                {
                    keyword = true
                }
                T::For | T::In if depth == 1 && i > self.pos + 1 => comp = true,
                _ => {}
            }
        }
        None
    }
}

/// Finds the token after the bracket group that opens at `tokens[open]`.
fn after_group(tokens: &[Token], open: usize) -> usize {
    let mut depth = 0u32;
    for (i, t) in tokens.iter().enumerate().skip(open) {
        match t.kind {
            T::LParen | T::LBracket | T::LBrace => depth += 1,
            T::RParen | T::RBracket | T::RBrace => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            T::Newline | T::Indent | T::Dedent | T::Eof => break,
            _ => {}
        }
    }
    tokens.len()
}

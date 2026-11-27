//! The parser: recursive descent over the lossless tokens, building the
//! green tree directly. Trivia go between nodes, never at a node's edge.
//! Recovery is per construct (handbook/design/next/syntax.md): a token
//! that cannot start what a construct wants becomes an `Error` node, or a
//! zero-width `Missing` node when it belongs to an enclosing construct, and
//! each of those carries exactly one diagnostic.

mod expr;
mod pattern;

use crate::file::{NodeKind, SyntaxFile};
use crate::lexer::lex;
use crate::token::{Token, TokenKind};
use crate::tree::{Checkpoint, GreenBuilder, SyntaxNode};
use cpl_diag::{codes, msgs, Code, Diagnostic, Span};
use TokenKind as T;

/// The tree of `src` read as one expression: a `File` holding the
/// expression and anything after it in an `Error` node. Total, like
/// [`crate::parse`].
pub fn parse_expr(src: &str) -> SyntaxFile {
    let lexed = lex(src);
    let tokens = lexed.lossless(src);
    let mut p = Parser::new(src, &tokens);
    p.start(NodeKind::File);
    p.expr();
    if !matches!(p.peek(), T::Eof | T::Newline) {
        p.error_here(codes::UNEXPECTED_TOKEN, "the end of the expression");
        p.start(NodeKind::Error);
        while !p.at(T::Eof) {
            p.bump();
        }
        p.finish();
    }
    while p.pos < p.tokens.len() {
        p.bump();
    }
    p.finish();
    let mut diagnostics = lexed.diagnostics;
    diagnostics.extend(p.diagnostics);
    SyntaxFile {
        root: SyntaxNode::new_root(p.b.finish()),
        diagnostics,
    }
}

/// Tokens that end the construct being read for every enclosing one: a
/// construct that meets one of these leaves it to its parent, with a
/// `Missing` child for what it lacked.
const CLOSERS: &[TokenKind] = &[
    T::RParen,
    T::RBracket,
    T::RBrace,
    T::Comma,
    T::Colon,
    T::Semi,
    T::Eq,
    T::FatArrow,
    T::Arrow,
    T::Newline,
    T::Indent,
    T::Dedent,
    T::Eof,
];

pub(crate) struct Parser<'t> {
    src: &'t str,
    tokens: &'t [Token],
    /// The next significant token; the trivia before it, from `emitted`,
    /// are not in the tree yet.
    pos: usize,
    emitted: usize,
    b: GreenBuilder,
    diagnostics: Vec<Diagnostic>,
}

impl<'t> Parser<'t> {
    fn new(src: &'t str, tokens: &'t [Token]) -> Parser<'t> {
        let mut p = Parser {
            src,
            tokens,
            pos: 0,
            emitted: 0,
            b: GreenBuilder::default(),
            diagnostics: Vec::new(),
        };
        p.skip_trivia();
        p
    }

    fn skip_trivia(&mut self) {
        while self
            .tokens
            .get(self.pos)
            .is_some_and(|t| t.kind.is_trivia())
        {
            self.pos += 1;
        }
    }

    /// Puts the trivia before the next token in the tree, outside any node
    /// about to start.
    fn flush(&mut self) {
        for t in &self.tokens[self.emitted..self.pos] {
            self.b.token(t.kind, t.span.text(self.src));
        }
        self.emitted = self.pos;
    }

    fn nth_index(&self, n: usize) -> Option<usize> {
        (self.pos..self.tokens.len())
            .filter(|&i| !self.tokens[i].kind.is_trivia())
            .nth(n)
    }

    fn nth(&self, n: usize) -> TokenKind {
        self.nth_index(n).map_or(T::Eof, |i| self.tokens[i].kind)
    }

    fn peek(&self) -> TokenKind {
        self.nth(0)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.peek() == kind
    }

    fn text(&self) -> &'t str {
        self.tokens
            .get(self.pos)
            .map_or("", |t| t.span.text(self.src))
    }

    fn span(&self) -> Span {
        self.tokens
            .get(self.pos)
            .map_or(Span::point(self.src.len() as u32), |t| t.span)
    }

    /// True when the token `n` ahead starts where the one before it ends.
    fn glued(&self, n: usize) -> bool {
        match (self.nth_index(n.wrapping_sub(1)), self.nth_index(n)) {
            (Some(a), Some(b)) if n > 0 => self.tokens[a].span.end == self.tokens[b].span.start,
            _ => false,
        }
    }

    fn bump(&mut self) {
        self.flush();
        if let Some(t) = self.tokens.get(self.pos) {
            self.b.token(t.kind, t.span.text(self.src));
            self.pos += 1;
            self.emitted = self.pos;
        }
        self.skip_trivia();
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        let hit = self.at(kind);
        if hit {
            self.bump();
        }
        hit
    }

    fn start(&mut self, kind: NodeKind) {
        self.flush();
        self.b.start_node(kind);
    }

    fn checkpoint(&mut self) -> Checkpoint {
        self.flush();
        self.b.checkpoint()
    }

    fn start_at(&mut self, at: Checkpoint, kind: NodeKind) {
        self.b.start_node_at(at, kind);
    }

    fn finish(&mut self) {
        self.b.finish_node();
    }

    fn error_here(&mut self, code: Code, wanted: &str) {
        let mut diag =
            msgs::syntax::EXPECTED_FOUND.error(self.span(), &[&wanted, &self.peek().describe()]);
        diag.code = code;
        self.diagnostics.push(diag);
    }

    /// What a construct wanted is not here: a `Missing` node when the token
    /// belongs to an enclosing construct, else an `Error` node holding it.
    /// One diagnostic either way.
    fn recover(&mut self, code: Code, wanted: &str) {
        self.error_here(code, wanted);
        if CLOSERS.contains(&self.peek()) || self.peek().starts_statement() {
            self.start(NodeKind::Missing);
        } else {
            self.start(NodeKind::Error);
            self.bump();
        }
        self.finish();
    }

    /// `kind`, or a `Missing` node and one diagnostic.
    fn want(&mut self, kind: TokenKind, wanted: &str) -> bool {
        if self.eat(kind) {
            return true;
        }
        self.error_here(codes::EXPECTED_TOKEN, wanted);
        self.start(NodeKind::Missing);
        self.finish();
        false
    }

    /// `open item, item, ... close` as a `node`. An element that fails
    /// ends at the next `,` or `close`, and the elements after it parse as
    /// usual.
    fn list(
        &mut self,
        node: NodeKind,
        close: TokenKind,
        what: &str,
        mut item: impl FnMut(&mut Self),
    ) {
        self.start(node);
        self.bump();
        while !self.at(close) && !matches!(self.peek(), T::Eof | T::Dedent | T::Newline) {
            let before = self.pos;
            item(self);
            if self.pos == before && !self.at(T::Comma) && !self.at(close) {
                self.start(NodeKind::Error);
                while !matches!(self.peek(), T::Comma | T::Eof | T::Dedent | T::Newline)
                    && !self.at(close)
                {
                    self.bump();
                }
                self.finish();
            }
            if !self.eat(T::Comma) {
                break;
            }
        }
        let closer = close.describe();
        self.want(close, &format!("{closer} to close {what}"));
        self.finish();
    }
}
#[cfg(test)]
mod tests;

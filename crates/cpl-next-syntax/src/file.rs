//! A file's tree: its items, the trivia between them and its end.

use crate::items::split_items;
use crate::lexer::lex;
use crate::token::{Token, TokenKind};
use crate::tree::{GreenBuilder, SyntaxNode};
use cpl_diag::Diagnostic;

/// The kinds of node in the tree.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum NodeKind {
    /// The whole file: its items, the trivia between them and `Eof`.
    File,
    /// One top-level item (`split_items`): the comment lines directly above
    /// it, then its first token to its last.
    Item,
    /// Bytes the parser skipped. Already reported.
    Error,
}

/// One file's syntax.
#[derive(Clone, Debug)]
pub struct SyntaxFile {
    pub root: SyntaxNode,
    /// What was wrong with the file's text, in file positions.
    pub diagnostics: Vec<Diagnostic>,
}

impl SyntaxFile {
    /// The file's items, in order.
    pub fn items(&self) -> impl Iterator<Item = SyntaxNode> + '_ {
        self.root.children().filter(|n| n.kind() == NodeKind::Item)
    }
}

/// Where the comments that belong to the next item start in the trivia
/// before it: the comment lines directly above it, with no blank line
/// between them or below the last. They move and format with the item, and
/// its docs are read from them.
fn attached_comments(src: &str, before: &[&Token]) -> usize {
    let mut k = before.len();
    while k >= 2
        && before[k - 1].kind == TokenKind::Whitespace
        && before[k - 1].span.text(src).matches('\n').count() == 1
        && before[k - 2].kind == TokenKind::Comment
    {
        k -= 2;
    }
    k
}

/// The tree of `src`. Total: any text gives a tree whose text is `src`.
pub fn parse(src: &str) -> SyntaxFile {
    let lexed = lex(src);
    let items = split_items(&lexed.tokens);
    let tokens = lexed.lossless(src);
    let mut b = GreenBuilder::default();
    b.start_node(NodeKind::File);
    let push = |b: &mut GreenBuilder, t: &Token| b.token(t.kind, t.span.text(src));
    let mut all = tokens.iter().peekable();
    // Index of the next non-trivia token, as `split_items` counts them.
    let mut next = 0usize;
    for item in &items {
        let (first, last) = item.tokens;
        let mut before = Vec::new();
        while let Some(t) = all.next_if(|t| next < first || t.kind.is_trivia()) {
            next += usize::from(!t.kind.is_trivia());
            before.push(t);
        }
        let docs = attached_comments(src, &before);
        for t in &before[..docs] {
            push(&mut b, t);
        }
        b.start_node(NodeKind::Item);
        for t in &before[docs..] {
            push(&mut b, t);
        }
        while let Some(t) = all.next_if(|_| next < last) {
            next += usize::from(!t.kind.is_trivia());
            push(&mut b, t);
        }
        b.finish_node();
    }
    for t in all {
        push(&mut b, t);
    }
    b.finish_node();
    SyntaxFile {
        root: SyntaxNode::new_root(b.finish()),
        diagnostics: lexed.diagnostics,
    }
}

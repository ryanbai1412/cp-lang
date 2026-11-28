//! Questions about everything under a node, answered by filtering its descendants: the
//! lossless tree needs no walker. The old pipeline's `cpl_syntax::visit` stays frozen with it.

use crate::ast::{AstNode, Expr, ForStmt, Pattern, WhileStmt};
use crate::fall_through::written_span;
use crate::file::NodeKind;
use crate::token::TokenKind;
use crate::tree::{SyntaxElement, SyntaxNode, SyntaxToken};
use cpl_diag::Span;

/// Whether anything under `node`, `node` included, names one of `names` as a value.
pub fn mentions_any(node: &SyntaxNode, names: &[&str]) -> bool {
    std::iter::once(node.clone())
        .chain(node.descendants())
        .filter_map(Expr::cast)
        .any(|e| matches!(e, Expr::Name(n) if n.ident().is_some_and(|t| names.contains(&t.text()))))
}

/// The names `pat` binds, in order. A one-word pattern binds its word unless it starts
/// with a capital, which names a variant; `self` binds `self`.
pub fn pattern_names(pat: &Pattern, out: &mut Vec<String>) {
    match pat {
        Pattern::Path(p) => match p.binding() {
            Some(word) if binds(&word) => out.push(word.text().to_owned()),
            _ => p
                .args()
                .iter()
                .flat_map(|a| a.patterns())
                .for_each(|p| pattern_names(&p, out)),
        },
        Pattern::Tuple(t) => t.patterns().for_each(|p| pattern_names(&p, out)),
        Pattern::Paren(p) => p.pattern().iter().for_each(|p| pattern_names(p, out)),
        Pattern::Record(r) => r.fields().for_each(|f| match f.pattern() {
            Some(p) => pattern_names(&p, out),
            None => out.extend(f.name().map(|n| n.text().to_owned())),
        }),
        // Each alternative binds the same names.
        Pattern::Or(o) => o
            .alternatives()
            .take(1)
            .for_each(|p| pattern_names(&p, out)),
        Pattern::Wildcard(_) | Pattern::Literal(_) => {}
    }
}

fn binds(word: &SyntaxToken) -> bool {
    word.kind() == TokenKind::SelfKw || !word.text().starts_with(|c: char| c.is_ascii_uppercase())
}

/// Every `return`, and every `break` or `continue` aimed at a loop outside `e`, that leaves
/// `e`, in order: ones inside a nested fn or lambda leave that instead.
pub fn leaves_out_of(e: &Expr) -> Vec<Span> {
    let top = e.syntax();
    top.descendants()
        .filter(|n| leaves_out(n, top))
        .map(|n| written_span(&n))
        .collect()
}

fn leaves_out(node: &SyntaxNode, top: &SyntaxNode) -> bool {
    let aim = match node.kind() {
        NodeKind::ReturnStmt => None,
        NodeKind::BreakStmt | NodeKind::ContinueStmt => Some(label(node)),
        _ => return false,
    };
    let mut child = node.clone();
    for up in node.ancestors() {
        if &child == top {
            break;
        }
        match up.kind() {
            NodeKind::FnDecl | NodeKind::LambdaExpr => return false,
            NodeKind::WhileStmt | NodeKind::ForStmt if child.kind() == NodeKind::Block => {
                if let Some(aim) = &aim {
                    let name = WhileStmt::cast(up.clone())
                        .and_then(|w| w.label())
                        .or_else(|| ForStmt::cast(up.clone()).and_then(|f| f.label()))
                        .map(|l| l.text().to_owned());
                    if aim.is_none() || *aim == name {
                        return false;
                    }
                }
            }
            _ => {}
        }
        child = up;
    }
    true
}

fn label(node: &SyntaxNode) -> Option<String> {
    node.children_with_tokens()
        .filter_map(|e| match e {
            SyntaxElement::Token(t) => Some(t),
            SyntaxElement::Node(_) => None,
        })
        .find(|t| t.kind() == TokenKind::Label)
        .map(|t| t.text().to_owned())
}

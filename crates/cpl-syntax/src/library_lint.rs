//! `@lint(name, from: "<code>", to: "<code>")` on a library fn (R06.4.9):
//! the hand-written code the fn replaces and the call that replaces it. The
//! fn's parameter names are the holes. The checker reports the declarations
//! it cannot use (E0505); `cpl-lint` matches the rest.

use crate::ast::{ExprKind, Stmt, StmtKind, StrPiece};
use crate::token::{Token, TokenKind};
use crate::ParsedItem;
use cpl_diag::{msgs, Diagnostic, Span};
use std::fmt::Display;

/// One usable `@lint` declaration.
#[derive(Clone, Debug)]
pub struct LibraryLint {
    pub name: String,
    /// The decorated fn.
    pub fn_name: String,
    /// The fn's parameter names.
    pub holes: Vec<String>,
    pub from: String,
    pub to: String,
    /// The decorator, in file positions.
    pub decorator: Span,
    /// The decorated fn, in file positions: the pattern does not apply
    /// inside it.
    pub item: Span,
}

/// The `@lint` declarations on `stmt`, an item at `offset`, and the errors
/// for those that cannot be used.
pub fn declared(stmt: &Stmt, offset: u32) -> (Vec<LibraryLint>, Vec<Diagnostic>) {
    let (mut lints, mut errors) = (Vec::new(), Vec::new());
    for d in stmt
        .decorators
        .iter()
        .filter(|d| !d.directive && d.name.text == "lint")
    {
        let span = d.span.shift(offset);
        let mut label: Option<(Span, String)> = None;
        let mut error = |msg: &msgs::Msg,
                         args: &[&dyn Display],
                         help: &str,
                         label: &mut Option<(Span, String)>| {
            let mut diag = msg.error(span, args).with_help(help.to_string());
            if let Some((at, text)) = label.take() {
                diag = diag.with_label(at, text);
            }
            errors.push(diag);
        };
        let StmtKind::Fn(f) = &stmt.kind else {
            error(
                &msgs::syntax::LINT_NOT_ON_FN,
                &[],
                "put it on the fn whose call the lint suggests",
                &mut label,
            );
            continue;
        };
        let text = |key: &str| {
            d.args
                .iter()
                .find(|a| a.name.as_ref().is_some_and(|n| n.text == key))
                .and_then(|a| match &a.value.kind {
                    ExprKind::Str(pieces) => match pieces.as_slice() {
                        [StrPiece::Text(t)] => Some(t.clone()),
                        [] => Some(String::new()),
                        _ => None,
                    },
                    _ => None,
                })
        };
        let name = d
            .args
            .iter()
            .find(|a| a.name.is_none())
            .and_then(|a| match &a.value.kind {
                ExprKind::Name(n) => Some(n.clone()),
                _ => None,
            });
        let (Some(name), Some(from), Some(to)) = (name, text("from"), text("to")) else {
            error(
                &msgs::syntax::LINT_ARGS,
                &[],
                "write `@lint(my_name, from: \"x * x\", to: \"sq(x)\")`; `from` and `to` are plain strings",
                &mut label,
            );
            continue;
        };
        let holes: Vec<String> = f.params.iter().map(|p| p.name.text.clone()).collect();
        let (Some((from_toks, _)), Some(to_toks)) = (run(&from), code(&to)) else {
            let msg = if run(&from).is_none() {
                &msgs::syntax::LINT_FROM_NOT_CODE
            } else {
                &msgs::syntax::LINT_TO_NOT_CODE
            };
            error(
                msg,
                &[&name],
                "write CP Lang code that parses on its own, such as `x * x`",
                &mut label,
            );
            continue;
        };
        let names = |toks: &[(TokenKind, String)]| -> Vec<String> {
            toks.iter()
                .filter(|(k, t)| is_name(*k, t))
                .map(|(_, t)| t.clone())
                .collect()
        };
        let (in_from, in_to) = (names(&from_toks), names(&to_toks));
        if let Some(p) = f.params.iter().find(|p| {
            let h = &p.name.text;
            in_from.contains(h) && !in_to.contains(h)
        }) {
            let h = &p.name.text;
            label = Some((p.name.span.shift(offset), format!("`{h}` is declared here")));
            error(
                &msgs::syntax::LINT_TO_DROPS,
                &[&name, &h],
                "use every parameter `from` matches in `to`, so the fix keeps what the code computes",
                &mut label,
            );
            continue;
        }
        if let Some(p) = f.params.iter().find(|p| {
            let h = &p.name.text;
            in_to.contains(h) && !in_from.contains(h)
        }) {
            let h = &p.name.text;
            label = Some((p.name.span.shift(offset), format!("`{h}` is declared here")));
            error(
                &msgs::syntax::LINT_TO_EXTRA,
                &[&name, &h],
                "`to` can use only the parameters that `from` matches",
                &mut label,
            );
            continue;
        }
        let calls = to_toks.windows(2).any(|w| {
            w[0].0 == TokenKind::Ident && w[0].1 == f.name.text && w[1].0 == TokenKind::LParen
        });
        if !calls {
            error(
                &msgs::syntax::LINT_TO_NO_CALL,
                &[&name, &f.name.text],
                "a library lint suggests the fn it decorates",
                &mut label,
            );
            continue;
        }
        lints.push(LibraryLint {
            name,
            fn_name: f.name.text.clone(),
            holes,
            from,
            to,
            decorator: span,
            item: stmt.span.shift(offset),
        });
    }
    (lints, errors)
}

/// The errors for the `@lint`s of one module, given as its items and their
/// offsets: those [`declared`] reports, and a name declared twice.
pub fn module_errors<'a>(
    items: impl IntoIterator<Item = (u32, &'a ParsedItem)>,
) -> Vec<Diagnostic> {
    let (mut first, mut errors): (Vec<(String, Span)>, Vec<Diagnostic>) = (Vec::new(), Vec::new());
    for (offset, item) in items {
        for stmt in &item.stmts {
            let (lints, found) = declared(stmt, offset);
            errors.extend(found);
            for lint in lints {
                match first.iter().find(|(name, _)| *name == lint.name) {
                    Some((_, at)) => errors.push(
                        msgs::syntax::LINT_TWICE
                            .error(lint.decorator, &[&lint.name])
                            .with_help("give each lint its own name")
                            .with_label(*at, "first declared here"),
                    ),
                    None => first.push((lint.name, lint.decorator)),
                }
            }
        }
    }
    errors
}

/// A name, or `self`: what a hole can be.
pub fn is_name(kind: TokenKind, text: &str) -> bool {
    kind == TokenKind::Ident || (kind == TokenKind::SelfKw && text == "self")
}

/// A pattern's tokens with their text.
pub type Tokens = Vec<(TokenKind, String)>;

/// The tokens of `src` if it parses as one expression or a run of
/// statements (a `from` may span consecutive statements of one block), and
/// the index in them where each statement ends.
pub fn run(src: &str) -> Option<(Tokens, Vec<usize>)> {
    let parsed = crate::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return None;
    }
    let all = tokens(src);
    let (mut toks, mut ends) = (Vec::new(), Vec::new());
    for (offset, item) in &parsed.items {
        for s in &item.stmts {
            let span = s.span.shift(*offset);
            for t in all.iter().filter(|t| span.contains(t.span)) {
                let text = &src[t.span.start as usize..t.span.end as usize];
                toks.push((t.kind, text.to_string()));
            }
            ends.push(toks.len());
        }
    }
    (!ends.is_empty()).then_some((toks, ends))
}

/// The tokens of `src` if it is one expression or statement, without the
/// line and block tokens.
pub fn code(src: &str) -> Option<Vec<(TokenKind, String)>> {
    let (toks, ends) = run(src)?;
    (ends.len() == 1).then_some(toks)
}

/// The tokens of `src` a pattern compares: no line, block or end tokens.
pub fn tokens(src: &str) -> Vec<Token> {
    crate::lex(src)
        .tokens
        .into_iter()
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof
            )
        })
        .collect()
}

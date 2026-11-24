//! `@lint(name, from: "<code>", to: "<code>")` on a library fn (R06.4.9):
//! the hand-written code the fn replaces and the call that replaces it. The
//! fn's parameter names are the holes. The checker reports the declarations
//! it cannot use (E0505); `cpl-lint` matches the rest.

use crate::ast::{EnumDecl, ExprKind, Stmt, StmtKind, StrPiece, StructDecl};
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
    /// The fn's parameter names, then the names `holes:` adds.
    pub holes: Vec<String>,
    /// How many of `holes` are the fn's parameters; `to` may drop the rest.
    pub params: usize,
    /// `when:`'s bounds: a hole and the least and greatest value the code
    /// it matched must be proven to take.
    pub when: Vec<Bound>,
    pub from: String,
    pub to: String,
    /// The decorator, in file positions.
    pub decorator: Span,
    /// The decorated fn, in file positions: the pattern does not apply
    /// inside it.
    pub item: Span,
}

/// `hole >= n`, `hole > n`, `hole <= n` or `hole < n` from `when:`, as the
/// inclusive range it asks the hole's value to lie in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bound {
    pub hole: String,
    pub lo: Option<i128>,
    pub hi: Option<i128>,
}

/// `when:`'s comma-separated bounds, if each is a name compared with an
/// integer literal.
pub fn bounds(src: &str) -> Option<Vec<Bound>> {
    let src = src.trim();
    if src.is_empty() {
        return Some(Vec::new());
    }
    src.split(',')
        .map(|part| {
            let toks = tokens(part);
            let text = |t: &Token| &part[t.span.start as usize..t.span.end as usize];
            let (name, op, rest) = match toks.as_slice() {
                [n, op, rest @ ..] if is_name(n.kind, text(n)) => (text(n), op.kind, rest),
                _ => return None,
            };
            let n: i128 = match rest {
                [v] if v.kind == TokenKind::Int => text(v).replace('_', "").parse().ok()?,
                [m, v] if m.kind == TokenKind::Minus && v.kind == TokenKind::Int => {
                    -text(v).replace('_', "").parse::<i128>().ok()?
                }
                _ => return None,
            };
            let (lo, hi) = match op {
                TokenKind::GtEq => (Some(n), None),
                TokenKind::Gt => (Some(n + 1), None),
                TokenKind::LtEq => (None, Some(n)),
                TokenKind::Lt => (None, Some(n - 1)),
                _ => return None,
            };
            Some(Bound {
                hole: name.to_string(),
                lo,
                hi,
            })
        })
        .collect()
}

/// The `@lint` declarations on `stmt`, an item at `offset`, and on its
/// methods (a struct's or enum's), and the errors for those that cannot be
/// used.
pub fn declared(stmt: &Stmt, offset: u32) -> (Vec<LibraryLint>, Vec<Diagnostic>) {
    let (mut lints, mut errors) = declared_on(stmt, offset);
    let methods = match &stmt.kind {
        StmtKind::Struct(s) => s.methods.as_slice(),
        StmtKind::Enum(e) => e.methods.as_slice(),
        _ => &[],
    };
    for m in methods {
        let (l, e) = declared_on(m, offset);
        lints.extend(l);
        errors.extend(e);
    }
    (lints, errors)
}

fn declared_on(stmt: &Stmt, offset: u32) -> (Vec<LibraryLint>, Vec<Diagnostic>) {
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
        let mut holes: Vec<String> = f.params.iter().map(|p| p.name.text.clone()).collect();
        let params = holes.len();
        holes.extend(
            text("holes")
                .iter()
                .flat_map(|h| h.split(','))
                .map(|h| h.trim().to_string())
                .filter(|h| !h.is_empty()),
        );
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
        if let Some(h) = holes[params..].iter().find(|h| !in_from.contains(h)) {
            error(
                &msgs::syntax::LINT_HOLE_UNUSED,
                &[&name, &h],
                "list in `holes:` only names `from` uses",
                &mut label,
            );
            continue;
        }
        let when = text("when").map_or(Some(Vec::new()), |w| bounds(&w));
        let Some(when) = when.filter(|w| {
            w.iter()
                .all(|b| in_from.contains(&b.hole) && holes.contains(&b.hole))
        }) else {
            error(
                &msgs::syntax::LINT_WHEN,
                &[&name],
                "write `when: \"b >= 1\"`: holes `from` uses, each compared with an integer",
                &mut label,
            );
            continue;
        };
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
            (w[0].0 == TokenKind::Ident && w[0].1 == f.name.text && w[1].0 == TokenKind::LParen)
                || (w[0].0 == TokenKind::Dot && w[1].0 == TokenKind::Ident && w[1].1 == f.name.text)
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
            params,
            when,
            from,
            to,
            decorator: span,
            item: stmt.span.shift(offset),
        });
    }
    if let StmtKind::Struct(StructDecl { methods, .. }) | StmtKind::Enum(EnumDecl { methods, .. }) =
        &stmt.kind
    {
        for m in methods {
            let (more, errs) = declared(m, offset);
            lints.extend(more);
            errors.extend(errs);
        }
    }
    (lints, errors)
}

/// The errors for the `@lint`s of one module, given as its items and their
/// offsets: those [`declared`] reports, and a name declared on two fns (one
/// fn may declare a name more than once: each `from` is a spelling of it).
pub fn module_errors<'a>(
    items: impl IntoIterator<Item = (u32, &'a ParsedItem)>,
) -> Vec<Diagnostic> {
    let (mut first, mut errors): (Vec<(String, Span, Span)>, Vec<Diagnostic>) =
        (Vec::new(), Vec::new());
    for (offset, item) in items {
        for stmt in &item.stmts {
            let (lints, found) = declared(stmt, offset);
            errors.extend(found);
            for lint in lints {
                match first.iter().find(|(name, _, _)| *name == lint.name) {
                    Some((_, _, item)) if *item == lint.item => {}
                    Some((_, at, _)) => errors.push(
                        msgs::syntax::LINT_TWICE
                            .error(lint.decorator, &[&lint.name])
                            .with_help("give each lint its own name")
                            .with_label(*at, "first declared here"),
                    ),
                    None => first.push((lint.name, lint.decorator, lint.item)),
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

#[cfg(test)]
mod tests {
    fn errors(src: &str) -> Vec<String> {
        let parsed = crate::parse_file(src);
        (parsed.items.iter())
            .flat_map(|(off, item)| item.stmts.iter().map(|s| super::declared(s, *off)))
            .flat_map(|(_, errs)| errs.into_iter().map(|e| e.message))
            .collect()
    }

    #[test]
    fn one_fn_may_declare_a_lint_twice_but_two_fns_may_not() {
        let errors = |src: &str| {
            let parsed = crate::parse_file(src);
            let items = parsed.items.iter().map(|(off, item)| (*off, item));
            super::module_errors(items)
                .into_iter()
                .map(|e| e.message)
                .collect::<Vec<_>>()
        };
        let one = "@lint(by_hand, from: \"x * x\", to: \"sq(x)\")\n@lint(by_hand, from: \"x ** 2\", to: \"sq(x)\")\npub fn sq(x: Int) -> Int = x * x\n";
        assert!(errors(one).is_empty(), "{:?}", errors(one));
        let two = "@lint(by_hand, from: \"x * x\", to: \"sq(x)\")\npub fn sq(x: Int) -> Int = x * x\n@lint(by_hand, from: \"x * x * x\", to: \"cube(x)\")\npub fn cube(x: Int) -> Int = x * x * x\n";
        assert_eq!(errors(two).len(), 1);
    }

    #[test]
    fn when_bounds_a_hole_from_uses_by_an_integer() {
        let decl = |when: &str| {
            format!("@lint(up, from: \"(self + b - 1) / b\", to: \"up(self, b)\", when: \"{when}\")\npub fn up(self: Int, b: Int) -> Int = self\n")
        };
        assert!(errors(&decl("b >= 1, self < -2")).is_empty());
        let want = super::Bound {
            hole: "self".into(),
            lo: None,
            hi: Some(-3),
        };
        assert_eq!(super::bounds("b > 0, self < -2").unwrap()[1], want);
        for bad in ["c >= 1", "b >= n", "b == 1"] {
            assert_eq!(
                errors(&decl(bad)),
                ["`when:` of `@lint(up)` is not holes `from` uses compared with integers"],
                "{bad}"
            );
        }
    }

    #[test]
    fn a_struct_or_enum_method_declares_a_lint() {
        let lints = |src: &str| {
            let parsed = crate::parse_file(src);
            (parsed.items.iter())
                .flat_map(|(off, item)| item.stmts.iter().map(|s| super::declared(s, *off)))
                .flat_map(|(lints, _)| lints.into_iter().map(|l| (l.name, l.fn_name)))
                .collect::<Vec<_>>()
        };
        let on_struct = "pub struct Box:\n    v: Int\n    @lint(boxed, from: \"x * x\", to: \"Box.sq(x)\")\n    pub fn sq(x: Int) -> Int = x * x\n";
        assert_eq!(lints(on_struct), [("boxed".to_string(), "sq".to_string())]);
        let on_enum = "pub enum Sign:\n    Neg\n    Pos\n    @lint(signed, from: \"x < 0\", to: \"Sign.of(x)\")\n    pub fn of(x: Int) -> Sign = if x < 0: Neg else: Pos\n";
        assert_eq!(lints(on_enum), [("signed".to_string(), "of".to_string())]);
        let bad = "pub struct Box:\n    v: Int\n    @lint(boxed, from: \"x * x\", to: \"cube(x)\")\n    pub fn sq(x: Int) -> Int = x * x\n";
        assert_eq!(errors(bad).len(), 1, "{:?}", errors(bad));
    }

    #[test]
    fn holes_must_be_used_by_from_and_to_may_name_the_fn_as_a_getter() {
        let ok = "@lint(walk_by_hand, from: \"xs[n]\", to: \"xs.walk\", holes: \"n\")\npub fn walk(xs: List<Int>) -> Int = 0\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        let unused = "@lint(walk_by_hand, from: \"xs[0]\", to: \"xs.walk\", holes: \"n\")\npub fn walk(xs: List<Int>) -> Int = 0\n";
        assert_eq!(
            errors(unused),
            ["`holes:` of `@lint(walk_by_hand)` names `n`, which `from` does not use"]
        );
    }
}

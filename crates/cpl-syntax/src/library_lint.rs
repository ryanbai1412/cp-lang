//! `@lint(name, from: "<code>", to: "<code>")` on a library fn (R06.4.9):
//! the hand-written code the fn replaces and the call that replaces it. The
//! fn's parameter names are the holes. The checker reports the declarations
//! it cannot use (E0505); `cpl-lint` matches the rest.

use crate::ast::{
    BinOp, CmpOp, Cond, Expr, ExprKind, FnBody, FnDecl, Stmt, StmtKind, StrPiece, TypeKind,
};
use crate::ParsedItem;
use crate::{Token, TokenKind};
use cpl_diag::{msgs, Diagnostic, Span};
use std::fmt::Display;

/// One usable `@lint` declaration.
#[derive(Clone, Debug)]
pub struct LibraryLint {
    pub name: String,
    /// The decorated fn.
    pub fn_name: String,
    /// What `to` calls: the fn, or the compiler built-in `builtin:` names
    /// when the fn documents one (`[R, C].filled(x)` on a grid fn).
    pub suggests: String,
    /// `from` calls the fn and `to` does not: the lint drops a needless
    /// call of it (`s.last!` before `s.pop()`).
    pub drops: bool,
    /// The type whose method the fn is, when it takes `self`: the lint
    /// keeps a finding only where `self` matched a value of that type.
    pub receiver: Option<String>,
    /// The fn's parameter names, then the names `holes:` adds.
    pub holes: Vec<String>,
    /// How many of `holes` are the fn's parameters; `to` may drop the rest.
    pub params: usize,
    /// `when:`'s bounds: a hole and the least and greatest value the code
    /// it matched must be proven to take.
    pub when: Vec<Bound>,
    /// Per parameter: the built-in scalar type it is written as, which the
    /// code its hole matches must have.
    pub types: Vec<Option<String>>,
    pub from: String,
    /// `to`, or `advice:` when `advice` is set.
    pub to: String,
    /// `advice:` in place of `to`: code the finding names, which need not
    /// call the fn and is never applied as a fix.
    pub advice: bool,
    /// `also:`'s expressions, one per line: the lint fires only where one
    /// of them matches somewhere in the file, the holes and names `from`
    /// bound standing for what they matched.
    pub also: String,
    /// `reads:`, `p[k] -> p.cut(k)`: how the fix rewrites each later read
    /// of a name `from` declares; empty when it rewrites none.
    pub reads: String,
    /// `each:`, `x in xs`: a name `to` writes for an element of the hole
    /// `xs`, spelled as its singular (`vals` -> `val`); empty for none.
    pub each: String,
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
    let mut out = Vec::new();
    for part in src.split(',').filter(|p| !p.trim().is_empty()) {
        let toks = tokens(part);
        let text = |t: &Token| &part[t.span.start as usize..t.span.end as usize];
        let [n, op, rest @ ..] = toks.as_slice() else {
            return None;
        };
        let (neg, v) = match rest {
            [v] => (false, v),
            [m, v] if m.kind == TokenKind::Minus => (true, v),
            _ => return None,
        };
        if !is_name(n.kind, text(n)) || v.kind != TokenKind::Int {
            return None;
        }
        let v: i128 = text(v).replace('_', "").parse().ok()?;
        let v = if neg { -v } else { v };
        let (lo, hi) = match op.kind {
            TokenKind::GtEq => (Some(v), None),
            TokenKind::Gt => (Some(v + 1), None),
            TokenKind::LtEq => (None, Some(v)),
            TokenKind::Lt => (None, Some(v - 1)),
            _ => return None,
        };
        out.push(Bound {
            hole: text(n).to_string(),
            lo,
            hi,
        });
    }
    Some(out)
}

/// The `@lint` declarations on `stmt`, an item at `offset`, and on its
/// methods (a struct's or enum's), and the errors for those that cannot be
/// used.
pub fn declared(stmt: &Stmt, offset: u32) -> (Vec<LibraryLint>, Vec<Diagnostic>) {
    let (mut lints, mut errors) = declared_on(stmt, offset, None);
    let (ty, methods) = match &stmt.kind {
        StmtKind::Struct(s) => (Some(&s.name.text), s.methods.as_slice()),
        StmtKind::Enum(e) => (Some(&e.name.text), e.methods.as_slice()),
        // A trait's method is every implementing type's: no one receiver.
        StmtKind::Trait { body, .. } => (None, body.stmts.as_slice()),
        _ => (None, &[][..]),
    };
    for m in methods {
        let (l, e) = declared_on(m, offset, ty);
        lints.extend(l);
        errors.extend(e);
    }
    (lints, errors)
}

fn declared_on(
    stmt: &Stmt,
    offset: u32,
    ty: Option<&String>,
) -> (Vec<LibraryLint>, Vec<Diagnostic>) {
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
        let text = |key: &str| arg_text(d, key);
        let name = d
            .args
            .iter()
            .find(|a| a.name.is_none())
            .and_then(|a| match &a.value.kind {
                ExprKind::Name(n) => Some(n.clone()),
                _ => None,
            });
        let to = match (text("to"), text("advice")) {
            (Some(to), None) => Some((to, false)),
            (None, Some(advice)) => Some((advice, true)),
            _ => None,
        };
        let (Some(name), Some(from), Some((to, advice))) = (name, text("from"), to) else {
            error(
                &msgs::syntax::LINT_ARGS,
                &[],
                "write `@lint(my_name, from: \"x * x\", to: \"sq(x)\")` (or `advice:` for `to`); `from` and `to` are plain strings",
                &mut label,
            );
            continue;
        };
        let mut holes: Vec<String> = f.params.iter().map(|p| p.name.text.clone()).collect();
        let params = holes.len();
        for h in text("holes").unwrap_or_default().split(',') {
            if !h.trim().is_empty() {
                holes.push(h.trim().to_string());
            }
        }
        let also = text("also").unwrap_or_default();
        let reads = text("reads").unwrap_or_default();
        let read = (reads.split_once("->")).map(|(r, w)| (code(r.trim()), code(w.trim())));
        let read_toks = match read {
            None if reads.is_empty() => Vec::new(),
            Some((Some(r), Some(w))) => [r, w].concat(),
            _ => {
                error(
                    &msgs::syntax::LINT_READS,
                    &[&name],
                    "write `reads: \"p[k] -> p.cut(k)\"`: a read of a name `from` declares, `->`, what the fix writes for it",
                    &mut label,
                );
                continue;
            }
        };
        let each = text("each").unwrap_or_default();
        let each_ok = each.is_empty()
            || each.split_once(" in ").is_some_and(|(x, xs)| {
                let x_in_to = tokens(&to)
                    .iter()
                    .any(|t| &to[t.span.start as usize..t.span.end as usize] == x.trim());
                x_in_to
                    && !holes.iter().any(|h| h == x.trim())
                    && holes.iter().any(|h| h == xs.trim())
            });
        if !each_ok {
            error(
                &msgs::syntax::LINT_EACH,
                &[&name],
                "write `each: \"x in xs\"`: a name `to` writes that is no hole, `in`, and the hole whose element it names",
                &mut label,
            );
            continue;
        }
        let (from_run, to_run) = (run(&from), run(&to));
        let bad_to = from_run.is_some() && to_run.is_none();
        let read_of = reads.split_once("->").map_or("", |r| r.0.trim());
        let (Some((from_toks, _)), Some((to_toks, _)), Some((used_toks, _))) =
            (from_run, to_run, run(&format!("{from}\n{also}\n{read_of}")))
        else {
            let msg = if bad_to {
                &msgs::syntax::LINT_TO_NOT_CODE
            } else {
                &msgs::syntax::LINT_FROM_NOT_CODE
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
            let kinds: Vec<TokenKind> = toks.iter().map(|t| t.0).collect();
            (toks.iter().zip(labels(&kinds)))
                .filter(|((k, t), label)| is_name(*k, t) && !label)
                .map(|((_, t), _)| t.clone())
                .collect()
        };
        let (in_from, in_to) = (names(&from_toks), names(&to_toks));
        let wrong = f.params.iter().find_map(|p| {
            let h = &p.name.text;
            match (in_from.contains(h), in_to.contains(h)) {
                (true, false) if !advice => Some((p, &msgs::syntax::LINT_TO_DROPS, "use every parameter `from` matches in `to`, so the fix keeps what the code computes")),
                (false, true) => Some((p, &msgs::syntax::LINT_TO_EXTRA, "`to` can use only the parameters that `from` matches")),
                _ => None,
            }
        });
        if let Some((p, msg, help)) = wrong {
            let h = &p.name.text;
            label = Some((p.name.span.shift(offset), format!("`{h}` is declared here")));
            error(msg, &[&name, &h], help, &mut label);
            continue;
        }
        let used = names(&used_toks);
        if let Some(h) = (holes[params..].iter()).find(|h| !used.contains(h)) {
            error(
                &msgs::syntax::LINT_HOLE_UNUSED,
                &[&name, &h],
                "list in `holes:` only names `from` or `also:` uses",
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
        let suggests = text("builtin").unwrap_or_else(|| f.name.text.clone());
        let calls = |toks: &[(TokenKind, String)], name: &str| {
            toks.windows(2).any(|w| {
                (w[0].0 == TokenKind::Ident && w[0].1 == name && w[1].0 == TokenKind::LParen)
                    || (w[0].0 == TokenKind::Dot && w[1].0 == TokenKind::Ident && w[1].1 == name)
            })
        };
        let uses = calls(&to_toks, &suggests)
            || calls(&read_toks, &suggests)
            || writes_body_ops(f, &from, &to, &to_toks)
            || compound(&to_toks, &f.name.text)
            || walks_self(f, &to_toks);
        let drops = !uses && !advice && calls(&from_toks, &f.name.text);
        if !uses && !drops && !advice {
            error(
                &msgs::syntax::LINT_TO_NO_CALL,
                &[&name, &suggests],
                "a library lint suggests the fn it decorates, or drops a needless call of it",
                &mut label,
            );
            continue;
        }
        lints.push(LibraryLint {
            name,
            fn_name: f.name.text.clone(),
            suggests,
            drops,
            receiver: ty
                .filter(|_| f.params.iter().any(|p| p.name.text == "self"))
                .cloned(),
            holes,
            params,
            when,
            types: f.params.iter().map(scalar).collect(),
            from,
            to,
            advice,
            also,
            reads,
            each,
            decorator: span,
            item: stmt.span.shift(offset),
        });
    }
    (lints, errors)
}

/// The plain string a decorator's `key:` argument gives. Not inlined:
/// `declared_on` reads seven of them.
#[inline(never)]
fn arg_text(d: &crate::ast::Decorator, key: &str) -> Option<String> {
    let a = d
        .args
        .iter()
        .find(|a| a.name.as_ref().is_some_and(|n| n.text == key))?;
    match &a.value.kind {
        ExprKind::Str(pieces) => match pieces.as_slice() {
            [StrPiece::Text(t)] => Some(t.clone()),
            [] => Some(String::new()),
            _ => None,
        },
        _ => None,
    }
}

/// The types a hole's parameter can be written as for its hole to match
/// only code the checker gives that type (a range type is an `Int`).
pub const SCALARS: &[&str] = &["Int", "I128", "U64", "F64", "Bool", "Char", "Str"];

fn scalar(p: &crate::ast::Param) -> Option<String> {
    match &p.ty.as_ref()?.kind {
        TypeKind::Named { name, args } if args.is_empty() && SCALARS.contains(&&*name.text) => {
            Some(name.text.clone())
        }
        _ => None,
    }
}

/// Whether `f` is a sequence's `iter` and `to` walks `self` with `for`,
/// which runs `iter` (R07.3.1): `for x in self` on `Seq`'s `iter`.
fn walks_self(f: &FnDecl, to_toks: &[(TokenKind, String)]) -> bool {
    use TokenKind::{For, Ident, In, SelfKw};
    f.name.text == "iter"
        && (to_toks.windows(4))
            .any(|w| [w[0].0, w[1].0, w[2].0, w[3].0] == [For, Ident, In, SelfKw])
}

/// Whether `to` calls nothing, writes a form `f`'s one-expression body
/// writes (an operator, a comparison, a `let` condition), and no form
/// that neither the body nor `from` writes: the lint teaches the core form
/// `f` is defined by (`self % d` on `divmod`, whose body is
/// `(self / d, self % d)`; `while let w = e, w > y` on `is_some_and`, whose
/// body is `if let`, with the `>` that `from` wrote).
fn writes_body_ops(f: &FnDecl, from: &str, to: &str, to_toks: &[(TokenKind, String)]) -> bool {
    #[derive(PartialEq)]
    enum Form {
        Op(BinOp),
        Cmp(CmpOp),
        Let,
    }
    fn binds(c: &Cond) -> bool {
        match c {
            Cond::Let { .. } => true,
            Cond::Guard { and, .. } => binds(and),
            _ => false,
        }
    }
    struct Forms(Vec<Form>);
    impl crate::visit::Visitor for Forms {
        fn stmt(&mut self, s: &Stmt) {
            if let StmtKind::While { cond, .. } = &s.kind {
                if binds(cond) {
                    self.0.push(Form::Let);
                }
            }
        }
        fn expr(&mut self, e: &Expr) {
            match &e.kind {
                ExprKind::Binary { op, .. } => self.0.push(Form::Op(*op)),
                ExprKind::Compare { rest, .. } => {
                    self.0.extend(rest.iter().map(|(op, _)| Form::Cmp(*op)))
                }
                ExprKind::If { branches, .. } if branches.iter().any(|b| binds(&b.0)) => {
                    self.0.push(Form::Let)
                }
                _ => {}
            }
        }
    }
    fn forms(src: &str) -> Forms {
        let mut forms = Forms(Vec::new());
        for (_, item) in &crate::parse_file(src).items {
            for s in &item.stmts {
                crate::visit::walk_stmt(s, &mut forms);
            }
        }
        forms
    }
    let FnBody::Expr(body) = &f.body else {
        return false;
    };
    let calls = to_toks.windows(2).any(|w| {
        w[1].0 == TokenKind::LParen && matches!(w[0].0, TokenKind::Ident | TokenKind::RParen)
    });
    let mut have = Forms(Vec::new());
    crate::visit::walk_expr(body, &mut have);
    let (said, used) = (forms(from), forms(to));
    !calls
        && used.0.iter().any(|x| have.0.contains(x))
        && (used.0.iter()).all(|x| have.0.contains(x) || said.0.contains(x))
}

/// Whether `to` uses `f` as its compound assignment, `p f= e` (`min=`,
/// `max=`, R01.6.4).
pub fn compound(to: &[(TokenKind, String)], f: &str) -> bool {
    to.windows(3).any(|w| {
        w[0].0 != TokenKind::Dot
            && w[1].0 == TokenKind::Ident
            && w[1].1 == f
            && w[2].0 == TokenKind::Eq
    })
}

/// The errors for the `@lint`s of one module, given as its items and their
/// offsets: those [`declared`] reports, and one name declared twice with the
/// same `from` on one receiver. A name may be declared once per spelling, on
/// several fns when its fix calls one of them (`min=` or `max=`), and once
/// per receiver on methods (`pop_first` on `SortedBag` and `SortedSet`).
pub fn module_errors<'a>(
    items: impl IntoIterator<Item = (u32, &'a ParsedItem)>,
) -> Vec<Diagnostic> {
    type Seen = (String, Option<String>, Option<Tokens>, Span);
    let (mut first, mut errors): (Vec<Seen>, Vec<Diagnostic>) = (Vec::new(), Vec::new());
    for (offset, item) in items {
        for stmt in &item.stmts {
            let (lints, found) = declared(stmt, offset);
            errors.extend(found);
            for lint in lints {
                let from = run(&lint.from).map(|(toks, _)| toks);
                match first.iter().find(|(name, ty, f, _)| {
                    *name == lint.name && *ty == lint.receiver && *f == from
                }) {
                    Some((.., at)) => errors.push(
                        msgs::syntax::LINT_TWICE
                            .error(lint.decorator, &[&lint.name])
                            .with_help("give each spelling of a lint its own `from`")
                            .with_label(*at, "first declared here"),
                    ),
                    None => first.push((lint.name, lint.receiver, from, lint.decorator)),
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

/// Per token, whether it is a named argument's label (`from` in
/// `f(from: x)`): it names the parameter, so it is never a hole. A `fn`'s
/// own parameters (`fn go(u: Int)`) are names, not labels.
pub fn labels(kinds: &[TokenKind]) -> Vec<bool> {
    use TokenKind as K;
    let mut open = Vec::new();
    (0..kinds.len())
        .map(|i| {
            let label = kinds[i] == K::Ident
                && open.last() == Some(&Some(K::LParen))
                && kinds.get(i + 1) == Some(&K::Colon)
                && matches!(kinds[i - 1], K::LParen | K::Comma);
            let params = i >= 2 && kinds[i - 1] == K::Ident && kinds[i - 2] == K::Fn;
            match kinds[i] {
                K::LParen if params => open.push(None),
                K::LParen | K::LBracket | K::LBrace => open.push(Some(kinds[i])),
                K::RParen | K::RBracket | K::RBrace => {
                    open.pop();
                }
                _ => {}
            }
            label
        })
        .collect()
}

/// A pattern's tokens with their text.
pub type Tokens = Vec<(TokenKind, String)>;

/// The tokens of `src` if it parses as one expression or a run of
/// statements (a `from` may span consecutive statements of one block), and
/// the index in them where each statement ends.
pub fn run(src: &str) -> Option<(Tokens, Vec<usize>)> {
    let (toks, ends, _) = run_spans(src)?;
    let toks = (toks.iter())
        .map(|t| {
            (
                t.kind,
                src[t.span.start as usize..t.span.end as usize].to_string(),
            )
        })
        .collect();
    Some((toks, ends))
}

/// The indices in [`run`]'s tokens of each name that is a whole statement
/// (`body` in `if c:\n    body`): a hole there matches a run of statements.
pub fn statement_names(src: &str) -> Vec<usize> {
    struct Names(Vec<Span>);
    impl crate::visit::Visitor for Names {
        fn stmt(&mut self, stmt: &Stmt) {
            if let StmtKind::Expr(e) = &stmt.kind {
                if matches!(e.kind, ExprKind::Name(_)) {
                    self.0.push(e.span);
                }
            }
        }
    }
    let Some((toks, _, parsed)) = run_spans(src) else {
        return Vec::new();
    };
    let mut names = Names(Vec::new());
    for (offset, item) in &parsed.items {
        for s in &item.stmts {
            let before = names.0.len();
            crate::visit::walk_stmt(s, &mut names);
            for span in &mut names.0[before..] {
                *span = span.shift(*offset);
            }
        }
    }
    (toks.iter().enumerate())
        .filter(|(_, t)| names.0.contains(&t.span))
        .map(|(i, _)| i)
        .collect()
}

fn run_spans(src: &str) -> Option<(Vec<Token>, Vec<usize>, crate::ParsedFile)> {
    let parsed = crate::parse_file(src);
    if !parsed.diagnostics.is_empty() {
        return None;
    }
    let all = tokens(src);
    let (mut toks, mut ends) = (Vec::new(), Vec::new());
    for (offset, item) in &parsed.items {
        for s in &item.stmts {
            let span = s.span.shift(*offset);
            toks.extend(all.iter().filter(|t| span.contains(t.span)).cloned());
            ends.push(toks.len());
        }
    }
    (!ends.is_empty()).then_some((toks, ends, parsed))
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
    /// A named argument's label is no hole; a `fn`'s parameter is a name.
    /// Rule: R06.4.9.
    #[test]
    fn a_named_argument_label_is_no_hole() {
        let src = "fn go(u: Int):\n    f(from: u, u)";
        let toks = crate::lex(src).tokens;
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        let labelled: Vec<&str> = (toks.iter().zip(super::labels(&kinds)))
            .filter(|(_, l)| *l)
            .map(|(t, _)| &src[t.span.start as usize..t.span.end as usize])
            .collect();
        assert_eq!(labelled, ["from"]);
    }

    fn errors(src: &str) -> Vec<String> {
        let parsed = crate::parse_file(src);
        (parsed.items.iter())
            .flat_map(|(off, item)| item.stmts.iter().map(|s| super::declared(s, *off)))
            .flat_map(|(_, errs)| errs.into_iter().map(|e| e.message))
            .collect()
    }

    fn parse_lints(src: &str) -> Vec<super::LibraryLint> {
        let parsed = crate::parse_file(src);
        (parsed.items.iter())
            .flat_map(|(off, item)| item.stmts.iter().map(|s| super::declared(s, *off)))
            .flat_map(|(lints, _)| lints)
            .collect()
    }

    #[test]
    fn a_lint_has_one_from_per_spelling_on_any_of_its_fns() {
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
        assert!(errors(two).is_empty(), "{:?}", errors(two));
        let again = "@lint(by_hand, from: \"x * x\", to: \"sq(x)\")\npub fn sq(x: Int) -> Int = x * x\n@lint(by_hand, from: \"x  *  x\", to: \"sq2(x)\")\npub fn sq2(x: Int) -> Int = x * x\n";
        assert_eq!(errors(again).len(), 1);
        let keep = "@lint(keep_max, from: \"x = max(x, e)\", to: \"x max= e\", holes: \"x, e\")\npub fn max(a: Int, b: Int) -> Int = a\n";
        let ok = |src: &str| {
            crate::parse_file(src).items.iter().all(|(off, item)| {
                item.stmts
                    .iter()
                    .all(|s| super::declared(s, *off).1.is_empty())
            })
        };
        assert!(ok(keep));
        let methods = "struct A(v: Int):\n    @lint(by_hand, from: \"self.v * 2\", to: \"self.twice()\")\n    pub fn twice(self) -> Int = v * 2\n    @lint(by_hand, from: \"self.v * 3\", to: \"self.thrice()\")\n    pub fn thrice(self) -> Int = v * 3\nstruct B(v: Int):\n    @lint(by_hand, from: \"self.v * 2\", to: \"self.twice()\")\n    pub fn twice(self) -> Int = v * 2\n";
        assert!(errors(methods).is_empty(), "{:?}", errors(methods));
    }

    #[test]
    fn when_bounds_a_hole_from_uses_by_an_integer() {
        let decl = |when: &str| {
            format!(
                "@lint(up, from: \"(self + b - 1) / b\", to: \"up(self, b)\", when: \"{when}\")\npub fn up(self: Int, b: Int) -> Int = self\n"
            )
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
    fn builtin_lets_to_call_the_compiler_built_in_the_fn_documents() {
        let ok = "@lint(grid_by_hand, from: \"[[c for _ in 0..m] for _ in 0..n]\", to: \"[n, m].filled(c)\", holes: \"c, m, n\", builtin: \"filled\")\npub fn rows(g: List<Int>) -> Int = 0\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        let other = "@lint(grid_by_hand, from: \"[[c for _ in 0..m] for _ in 0..n]\", to: \"rows([n, m].filled(c))\", holes: \"c, m, n\", builtin: \"fill\")\npub fn rows(g: List<Int>) -> Int = 0\n";
        assert_eq!(
            errors(other),
            ["`to` of `@lint(grid_by_hand)` does not call `fill`"]
        );
    }

    #[test]
    fn advice_need_not_call_the_fn_and_also_may_use_a_hole() {
        let ok = "@lint(probe, from: \"var t = v\", advice: \"var t = None\", also: \"t[k] == v\\nv == t[k]\", holes: \"k\")\npub fn probe(v: Int) = ()\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        let lints = parse_lints(ok);
        assert!(lints[0].advice);
        assert_eq!(lints[0].also, "t[k] == v\nv == t[k]");
        let both = "@lint(probe, from: \"var t = v\", to: \"probe(v)\", advice: \"var t = None\")\npub fn probe(v: Int) = ()\n";
        assert_eq!(errors(both).len(), 1);
        let bad_also = "@lint(probe, from: \"var t = v\", advice: \"var t = None\", also: \"t[\")\npub fn probe(v: Int) = ()\n";
        assert_eq!(errors(bad_also).len(), 1);
    }

    #[test]
    fn holes_must_be_used_by_from_and_to_may_name_the_fn_as_a_getter() {
        let ok = "@lint(walk_by_hand, from: \"xs[n]\", to: \"xs.walk\", holes: \"n\")\npub fn walk(xs: List<Int>) -> Int = 0\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        let unused = "@lint(walk_by_hand, from: \"xs[0]\", to: \"xs.walk\", holes: \"n\")\npub fn walk(xs: List<Int>) -> Int = 0\n";
        assert_eq!(
            errors(unused),
            ["`holes:` of `@lint(walk_by_hand)` names `n`, which neither `from` nor `also:` uses"]
        );
    }

    #[test]
    fn a_traits_method_declares_a_lint_for_every_type() {
        let src = "pub trait Walk<T>:\n    @lint(walk_by_hand, from: \"self.walk_all()\", to: \"self.walk()\")\n    fn walk(self) -> yields T\n";
        let lints = parse_lints(src);
        assert_eq!(lints.len(), 1, "{:?}", errors(src));
        assert_eq!(
            (lints[0].fn_name.as_str(), &lints[0].receiver),
            ("walk", &None)
        );
    }

    #[test]
    fn a_to_that_walks_self_with_for_uses_iter() {
        let src = "pub trait Walk<T>:\n    @lint(index_walk, from: \"for i in 0..self.len:\\n    b\", to: \"for x in self:\\n    b\", holes: \"b\")\n    fn iter(self) -> yields T\n";
        assert!(errors(src).is_empty(), "{:?}", errors(src));
        let other = src.replace("fn iter", "fn walk");
        assert_eq!(errors(&other).len(), 1, "{:?}", errors(&other));
    }

    #[test]
    fn each_names_a_to_name_for_an_element_of_a_hole() {
        let ok = "pub trait Walk<T>:\n    @lint(index_walk, from: \"for i in 0..self.len:\\n    b\", to: \"for x in self:\\n    b\", each: \"x in self\", holes: \"b\")\n    fn iter(self) -> yields T\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        assert_eq!(parse_lints(ok)[0].each, "x in self");
        for bad in ["y in self", "x in q", "b in self", "x"] {
            let src = ok.replace("\"x in self\"", &format!("\"{bad}\""));
            assert_eq!(errors(&src).len(), 1, "{bad}: {:?}", errors(&src));
        }
    }

    #[test]
    fn reads_is_a_read_and_its_rewrite_and_may_call_the_fn() {
        let ok = "@lint(table_by_hand, from: \"var p = [0]\", to: \"let p = [1]\", reads: \"p[k] -> p.cut(k)\", holes: \"k\")\npub fn cut(t: Int, k: Int) -> Int = k\n";
        assert!(errors(ok).is_empty(), "{:?}", errors(ok));
        assert_eq!(parse_lints(ok)[0].reads, "p[k] -> p.cut(k)");
        let bad = "@lint(table_by_hand, from: \"var p = [0]\", to: \"let p = [1]\", reads: \"p[k]\", holes: \"k\")\npub fn cut(t: Int, k: Int) -> Int = k\n";
        assert_eq!(
            errors(bad),
            [
                "`reads:` of `@lint(table_by_hand)` is not a read, `->`, and what the fix writes for it"
            ]
        );
    }

    #[test]
    fn to_may_write_only_the_operators_of_the_fns_body() {
        let op = "@lint(rem_guard, from: \"(x + d) % d\", to: \"x % d\")\npub fn rem(x: Int, d: Int) -> Int = x % d\n";
        assert!(errors(op).is_empty(), "{:?}", errors(op));
        let other = "@lint(rem_guard, from: \"(x + d) % d\", to: \"x / d\")\npub fn rem(x: Int, d: Int) -> Int = x % d\n";
        let call = "@lint(rem_guard, from: \"(x + d) % d\", to: \"abs(x) % d\")\npub fn rem(x: Int, d: Int) -> Int = x % d\n";
        for bad in [other, call] {
            assert_eq!(errors(bad).len(), 1, "{bad}: {:?}", errors(bad));
        }
    }

    #[test]
    fn to_may_write_its_fns_let_condition_and_what_from_wrote() {
        let fun = "pub fn some_and(self: ?Int, p: Int -> Bool) -> Bool = if let Some(v) = self: p(v) else: false\n";
        let let_cond = format!("@lint(test_then_unwrap, from: \"if (self ?? c) > y:\\n    let w = self!\\n    body\", to: \"if let w = self, w > y:\\n    body\", holes: \"c, y, body\")\n{fun}");
        assert!(errors(&let_cond).is_empty(), "{:?}", errors(&let_cond));
        for bad in ["w + 1 > y:", "w < y:"] {
            let bad = let_cond.replace("w > y:", bad);
            assert_eq!(errors(&bad).len(), 1, "{bad}: {:?}", errors(&bad));
        }
        let no_form = let_cond.replace("if let w = self, w > y:\\n    body", "body");
        assert_eq!(errors(&no_form).len(), 1, "{:?}", errors(&no_form));
    }
}

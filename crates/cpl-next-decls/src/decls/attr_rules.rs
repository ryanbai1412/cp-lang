//! The rules a decorator follows beyond its name: what each `@name` may sit on and what its
//! arguments are. Collect calls `check` once per decorator, where forms reads it (R11.6).
use crate::attrs::{DeclKind, Decorator};
use crate::decls::forms::{AttrForm, FieldForm, FnForm, Form, TyForm};
use crate::decls::Role;
use crate::diag::Msg;
use crate::sym::{Sym, Syms};
use cpl_diag::{codes, msgs};
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast::{self, AstNode};
use cpl_next_syntax::grammar::{LintArg, StressKind};
use cpl_next_syntax::TokenKind;

/// The errors of the decorator `attr` on a declaration of kind `on` in `item`.
pub fn check(attr: &AttrForm, on: DeclKind, item: &ItemId, errors: &mut Vec<Msg>) {
    let Some(node) = &attr.node else {
        return;
    };
    let msg = match attr.attr {
        Decorator::Stress => stress(node, on),
        Decorator::Role => role(node, on),
        Decorator::Lint if !matches!(on, DeclKind::Fn { .. }) => Some(Msg {
            help: Some("put it on the fn whose call the lint suggests".to_string()),
            ..Msg::verdict(
                codes::BAD_LIBRARY_LINT,
                msgs::syntax::LINT_NOT_ON_FN.say(&[]),
            )
        }),
        _ => None,
    };
    errors.extend(msg.map(|m| Msg::at(item.at(node.syntax().span()), m)));
}

/// The errors of the decorators on a declaration other than a fn, and on its fields and
/// variants; a fn's are checked where its signature is read.
pub fn check_form(form: &Form, item: &ItemId, errors: &mut Vec<Msg>) {
    let mut on = |attrs: &[AttrForm], kind: DeclKind| {
        for a in attrs {
            check(a, kind, item, errors);
        }
    };
    let fields = |fs: &[FieldForm], on: &mut dyn FnMut(&[AttrForm], DeclKind)| {
        for f in fs {
            on(&f.attrs, DeclKind::Field);
        }
    };
    match form {
        Form::Struct(s) => {
            on(&s.attrs, DeclKind::Struct);
            fields(&s.fields, &mut on);
        }
        Form::Enum(e) => {
            on(&e.attrs, DeclKind::Enum);
            for v in &e.variants {
                on(&v.attrs, DeclKind::Variant);
                fields(&v.fields, &mut on);
            }
        }
        Form::Trait(t) => on(&t.attrs, DeclKind::Trait),
        Form::Impl(i) => on(&i.attrs, DeclKind::Impl),
        Form::Alias(a) => on(&a.attrs, DeclKind::Alias),
        Form::Context { fields: fs, .. } => fields(fs, &mut on),
        _ => {}
    }
}

fn args(node: &ast::Decorator) -> Vec<ast::Arg> {
    node.args().map(|l| l.args().collect()).unwrap_or_default()
}

/// The bare name an argument is, as `tree` in `@stress(tree)`.
fn bare_name(a: &ast::Arg) -> Option<String> {
    if a.label().is_some() {
        return None;
    }
    match a.value()? {
        ast::Expr::Name(n) => Some(n.ident()?.text().to_string()),
        _ => None,
    }
}

/// `@stress(kind)` sits on a type alias and names one kind (E0325).
fn stress(node: &ast::Decorator, on: DeclKind) -> Option<Msg> {
    let (text, help) = if on != DeclKind::Alias {
        (
            msgs::syntax::STRESS_NOT_ON_TYPE.say(&[]),
            "put it on a `type` alias, as in `@stress(tree) type T = [0..n - 1](u: V, v: V)`",
        )
    } else {
        let kind = match args(node).as_slice() {
            [a] => bare_name(a),
            _ => None,
        };
        if kind.is_some_and(|k| StressKind::of(&k).is_some()) {
            return None;
        }
        let kinds = StressKind::ROWS.map(|r| format!("`{}`", r.1)).join(", ");
        (
            msgs::syntax::STRESS_KIND.say(&[&kinds]),
            "write one bare kind, as in `@stress(tree)`",
        )
    };
    Some(Msg {
        help: Some(help.to_string()),
        ..Msg::verdict(codes::BAD_STRESS, text)
    })
}

/// `@role(trace)` is filled by a fn of one parameter: the value a trace shows (R11.6.1).
fn role(node: &ast::Decorator, on: DeclKind) -> Option<Msg> {
    let trace = args(node).first().and_then(bare_name).as_deref() == Some(Role::Trace.name());
    match on {
        DeclKind::Fn { params } if trace && params != 1 => Some(Msg::verdict(
            codes::TYPE_MISMATCH,
            format!("`@role(trace)` is filled by a fn of one parameter, the value a trace shows; this one takes {params}"),
        )),
        _ => None,
    }
}

/// What a `@lint` fn's declaration needs beyond the decorator: its name, its parameters'
/// names (`self` included) and whether it yields, and the tokens of a one-expression body.
pub struct LintedFn<'a> {
    pub name: &'a str,
    pub params: Vec<String>,
    pub yields: bool,
    pub body: Option<Vec<(TokenKind, String)>>,
}

/// `@lint(name, from: "<code>", to: "<code>")` on a fn (E0505, R06.4.9): the code the fn replaces
/// and the call that replaces it, the fn's parameter names being the holes.
pub fn lint(attr: &AttrForm, f: &LintedFn, item: &ItemId, errors: &mut Vec<Msg>) {
    let Some(node) = &attr.node else {
        return;
    };
    if let Some((text, help)) = lint_error(node, f) {
        let m = Msg {
            help: Some(help.to_string()),
            ..Msg::verdict(codes::BAD_LIBRARY_LINT, text)
        };
        errors.push(Msg::at(item.at(node.syntax().span()), m));
    }
}

/// The plain string a decorator's `key:` argument gives.
fn arg_text(node: &ast::Decorator, key: LintArg) -> Option<String> {
    let a = args(node).into_iter().find(|a| {
        a.label()
            .is_some_and(|l| LintArg::of(l.text()) == Some(key))
    })?;
    let ast::Expr::Literal(l) = a.value()? else {
        return None;
    };
    let t = l.token()?;
    let raw = t.text().strip_prefix('"')?.strip_suffix('"')?.to_string();
    if t.kind() != TokenKind::Str || raw.replace("{{", "").contains('{') {
        return None;
    }
    let mut out = String::new();
    let mut chars = raw
        .replace("{{", "{")
        .replace("}}", "}")
        .chars()
        .collect::<Vec<_>>()
        .into_iter();
    while let Some(c) = chars.next() {
        out.push(match (c, c == '\\') {
            (_, true) => match chars.next()? {
                'n' => '\n',
                't' => '\t',
                o => o,
            },
            (c, false) => c,
        });
    }
    Some(out)
}

/// The tokens of `src` a pattern compares: no line, block or end tokens.
fn tokens(src: &str) -> Vec<(TokenKind, String)> {
    (cpl_next_syntax::lex(src).tokens.into_iter())
        .filter(|t| {
            !matches!(
                t.kind,
                TokenKind::Newline | TokenKind::Indent | TokenKind::Dedent | TokenKind::Eof
            )
        })
        .map(|t| {
            (
                t.kind,
                src[t.span.start as usize..t.span.end as usize].to_string(),
            )
        })
        .collect()
}

/// The tokens of `src` and its statement count, if it parses as a run of statements.
fn run(src: &str) -> Option<(Vec<(TokenKind, String)>, usize)> {
    let file = cpl_next_syntax::parse_stmts(src);
    let n = file.root.children().count();
    (file.diagnostics.is_empty() && n > 0).then(|| (tokens(src), n))
}

/// The names a pattern's tokens use, labels aside: what a hole can be.
fn names(toks: &[(TokenKind, String)]) -> Vec<String> {
    let kinds: Vec<TokenKind> = toks.iter().map(|t| t.0).collect();
    (toks.iter().zip(cpl_next_syntax::arg_labels(&kinds)))
        .filter(|((k, _), label)| matches!(k, TokenKind::Ident | TokenKind::SelfKw) && !label)
        .map(|((_, t), _)| t.clone())
        .collect()
}

/// `when:`'s comma-separated bounds, if each is a hole compared with an integer literal.
fn bounds_ok(src: &str, holes: &[String], in_from: &[String]) -> bool {
    use TokenKind as K;
    src.split(',').filter(|p| !p.trim().is_empty()).all(|part| {
        let toks = tokens(part);
        let (n, op, v) = match toks.as_slice() {
            [n, op, v] => (n, op, v),
            [n, op, m, v] if m.0 == K::Minus => (n, op, v),
            _ => return false,
        };
        n.0 == K::Ident
            && v.0 == K::Int
            && matches!(op.0, K::Gt | K::GtEq | K::Lt | K::LtEq)
            && holes.contains(&n.1)
            && in_from.contains(&n.1)
    })
}

fn calls(toks: &[(TokenKind, String)], name: &str) -> bool {
    toks.windows(2).any(|w| {
        (w[0].0 == TokenKind::Ident && w[0].1 == name && w[1].0 == TokenKind::LParen)
            || (w[0].0 == TokenKind::Dot && w[1].0 == TokenKind::Ident && w[1].1 == name)
    })
}

/// The operator forms tokens write (`+`, `<`, a `let` condition): what a body-op lint teaches.
fn op_forms(toks: &[(TokenKind, String)]) -> Vec<String> {
    let op = |t: &str| !t.is_empty() && t != "=" && t.chars().all(|c| "+-*/%<>=!&|^".contains(c));
    (toks.iter())
        .filter(|(k, t)| op(t) || *k == TokenKind::Let)
        .map(|t| t.1.clone())
        .collect()
}

/// Why the `@lint` on `f` cannot be used, as its message and help; `None` when it can.
fn lint_error(node: &ast::Decorator, f: &LintedFn) -> Option<(String, &'static str)> {
    use msgs::syntax as m;
    let text = |key: LintArg| arg_text(node, key);
    let name = args(node)
        .iter()
        .find(|a| a.label().is_none())
        .and_then(bare_name);
    let to = match (text(LintArg::To), text(LintArg::Advice)) {
        (Some(to), None) => Some((to, false)),
        (None, Some(advice)) => Some((advice, true)),
        _ => None,
    };
    let (Some(name), Some(from), Some((to, advice))) = (name, text(LintArg::From), to) else {
        return Some((m::LINT_ARGS.say(&[]), "write `@lint(my_name, from: \"x * x\", to: \"sq(x)\")` (or `advice:` for `to`); `from` and `to` are plain strings"));
    };
    let mut holes = f.params.clone();
    holes.extend(
        text(LintArg::Holes)
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|h| !h.is_empty())
            .map(String::from),
    );
    let (also, reads, each) = (
        text(LintArg::Also).unwrap_or_default(),
        text(LintArg::Reads).unwrap_or_default(),
        text(LintArg::Each).unwrap_or_default(),
    );
    let code = |s: &str| run(s).filter(|(_, n)| *n == 1).map(|(t, _)| t);
    let read_toks = match reads.split_once("->").map(|(r, w)| (code(r.trim()), code(w.trim()))) {
        None if reads.is_empty() => Vec::new(),
        Some((Some(r), Some(w))) => [r, w].concat(),
        _ => return Some((m::LINT_READS.say(&[&name]), "write `reads: \"p[k] -> p.cut(k)\"`: a read of a name `from` declares, `->`, what the fix writes for it")),
    };
    let each_ok = each.is_empty()
        || each.split_once(" in ").is_some_and(|(x, xs)| {
            tokens(&to).iter().any(|t| t.1 == x.trim())
                && !holes.iter().any(|h| h == x.trim())
                && holes.iter().any(|h| h == xs.trim())
        });
    if !each_ok {
        return Some((m::LINT_EACH.say(&[&name]), "write `each: \"x in xs\"`: a name `to` writes that is no hole, `in`, and the hole whose element it names"));
    }
    let read_of = reads.split_once("->").map_or("", |r| r.0.trim());
    let (from_run, to_run) = (run(&from), run(&to));
    let bad_to = from_run.is_some() && to_run.is_none();
    let (Some((from_toks, _)), Some((to_toks, _)), Some((used_toks, _))) =
        (from_run, to_run, run(&format!("{from}\n{also}\n{read_of}")))
    else {
        let msg = if bad_to {
            &m::LINT_TO_NOT_CODE
        } else {
            &m::LINT_FROM_NOT_CODE
        };
        return Some((
            msg.say(&[&name]),
            "write CP Lang code that parses on its own, such as `x * x`",
        ));
    };
    let (in_from, in_to) = (names(&from_toks), names(&to_toks));
    for h in &f.params {
        match (in_from.contains(h), in_to.contains(h)) {
            (true, false) if !advice => return Some((m::LINT_TO_DROPS.say(&[&name, h]), "use every parameter `from` matches in `to`, so the fix keeps what the code computes")),
            (false, true) => return Some((m::LINT_TO_EXTRA.say(&[&name, h]), "`to` can use only the parameters that `from` matches")),
            _ => {}
        }
    }
    let used = names(&used_toks);
    if let Some(h) = holes[f.params.len()..].iter().find(|h| !used.contains(h)) {
        return Some((
            m::LINT_HOLE_UNUSED.say(&[&name, h]),
            "list in `holes:` only names `from` or `also:` uses",
        ));
    }
    if !text(LintArg::When).is_none_or(|w| bounds_ok(&w, &holes, &in_from)) {
        return Some((
            m::LINT_WHEN.say(&[&name]),
            "write `when: \"b >= 1\"`: holes `from` uses, each compared with an integer",
        ));
    }
    let suggests = text(LintArg::Builtin).unwrap_or_else(|| f.name.to_string());
    let compound = to_toks.windows(3).any(|w| {
        w[0].0 != TokenKind::Dot
            && w[1].0 == TokenKind::Ident
            && w[1].1 == f.name
            && w[2].0 == TokenKind::Eq
    });
    let walks_self = f.yields
        && (to_toks.windows(4)).any(|w| {
            [w[0].0, w[1].0, w[2].0, w[3].0]
                == [
                    TokenKind::For,
                    TokenKind::Ident,
                    TokenKind::In,
                    TokenKind::SelfKw,
                ]
        });
    let body_ops = f.body.as_ref().is_some_and(|body| {
        let (have, said, used) = (op_forms(body), op_forms(&from_toks), op_forms(&to_toks));
        let calls_any = to_toks.windows(2).any(|w| {
            w[1].0 == TokenKind::LParen && matches!(w[0].0, TokenKind::Ident | TokenKind::RParen)
        });
        !calls_any
            && used.iter().any(|x| have.contains(x))
            && used.iter().all(|x| have.contains(x) || said.contains(x))
    });
    let uses = calls(&to_toks, &suggests)
        || calls(&read_toks, &suggests)
        || body_ops
        || compound
        || walks_self;
    let drops = !uses && !advice && calls(&from_toks, f.name);
    (!uses && !drops && !advice).then(|| {
        (
            m::LINT_TO_NO_CALL.say(&[&name, &suggests]),
            "a library lint suggests the fn it decorates, or drops a needless call of it",
        )
    })
}

/// The `@lint`s on a module's fns and methods, checked by [`module_lints`]; a method's
/// receiver is the type whose body or impl it is in.
pub fn form_lints(forms: &[(ItemId, Form)], syms: &Syms, errors: &mut Vec<Msg>) {
    fn push<'a>(
        sites: &mut Vec<(&'a ItemId, &'a AttrForm, Option<String>)>,
        item: &'a ItemId,
        fns: &'a [FnForm],
        receiver: Option<String>,
    ) {
        for f in fns {
            for a in f.attrs.iter().filter(|a| a.attr == Decorator::Lint) {
                sites.push((item, a, receiver.clone()));
            }
        }
    }
    let mut sites = Vec::new();
    for (item, form) in forms {
        let named = |n: Sym| Some(syms.text(n).to_string());
        match form {
            Form::Fn(f) => push(&mut sites, item, std::slice::from_ref(f), None),
            Form::Struct(s) => push(&mut sites, item, &s.methods, named(s.name)),
            Form::Enum(e) => push(&mut sites, item, &e.methods, named(e.name)),
            Form::Trait(t) => push(&mut sites, item, &t.methods, named(t.name)),
            Form::Impl(i) => {
                let receiver = match &i.self_ty {
                    TyForm::Written(t) => Some(t.syntax().text().trim().to_string()),
                    _ => None,
                };
                push(&mut sites, item, &i.methods, receiver);
            }
            _ => {}
        }
    }
    module_lints(sites.iter().map(|(i, a, r)| (*i, *a, r.as_deref())), errors);
}

/// A module's `@lint`s, each with its item and the type whose method it is on: one name may
/// say the same `from` once per receiver (LINT_TWICE, E0505), as `pop_first` on two bags.
pub fn module_lints<'a>(
    lints: impl IntoIterator<Item = (&'a ItemId, &'a AttrForm, Option<&'a str>)>,
    errors: &mut Vec<Msg>,
) {
    type Seen<'a> = (
        String,
        Option<&'a str>,
        Vec<(TokenKind, String)>,
        cpl_next_ir::ids::SourceId,
    );
    let mut first: Vec<Seen> = Vec::new();
    for (item, attr, receiver) in lints {
        let Some(node) = &attr.node else {
            continue;
        };
        let name = args(node)
            .iter()
            .find(|a| a.label().is_none())
            .and_then(bare_name);
        let from = arg_text(node, LintArg::From)
            .and_then(|f| run(&f))
            .map(|(t, _)| t);
        let (Some(name), Some(from)) = (name, from) else {
            continue;
        };
        let at = item.at(node.syntax().span());
        match first
            .iter()
            .find(|(n, r, f, _)| *n == name && *r == receiver && *f == from)
        {
            Some((.., was)) => {
                let m = Msg {
                    help: Some("give each spelling of a lint its own `from`".to_string()),
                    labels: vec![(was.clone(), "first declared here".to_string())],
                    ..Msg::verdict(
                        codes::BAD_LIBRARY_LINT,
                        msgs::syntax::LINT_TWICE.say(&[&name]),
                    )
                };
                errors.push(Msg::at(at, m));
            }
            None => first.push((name, receiver, from, at)),
        }
    }
}

/// E0107 on a decorator no chapter documents (R06.4.11), with the nearest live one as help.
pub fn unknown(item: &ItemId, x: &ast::Decorator, errors: &mut Vec<Msg>) {
    let Some(n) = x.name() else {
        return;
    };
    let name = n.text();
    if Decorator::of(name).is_some() {
        return;
    }
    let live = cpl_next_syntax::grammar::DECORATORS.iter().map(|(d, _)| *d);
    let near = (live.clone())
        .filter(|d| cpl_diag::edit_distance(d, name) <= 2)
        .min_by_key(|d| cpl_diag::edit_distance(d, name));
    let help = match near {
        Some(d) => format!("did you mean `@{d}`?"),
        None => {
            let all: Vec<String> = live.map(|d| format!("`@{d}`")).collect();
            format!("the decorators are {}", all.join(", "))
        }
    };
    let m = Msg {
        help: Some(help),
        ..Msg::verdict(
            codes::UNEXPECTED_TOKEN,
            msgs::stmt::UNKNOWN_DECORATOR.say(&[&name]),
        )
    };
    errors.push(Msg::at(item.at(n.span()), m));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(src: &str, on: DeclKind) -> Vec<String> {
        let file = cpl_next_syntax::parse(src);
        let node = (file.root.descendants())
            .find_map(ast::Decorator::cast)
            .expect("a decorator");
        let attr = Decorator::live(node.name().expect("a name").text()).expect("an attribute");
        let mut out = Vec::new();
        let form = AttrForm {
            attr,
            node: Some(node),
        };
        check(
            &form,
            on,
            &ItemId::file("t.cp".into(), "t".into()),
            &mut out,
        );
        out.into_iter()
            .map(|m| format!("{}: {}", m.code.0, m.text))
            .collect()
    }

    #[test]
    fn stress_names_one_kind_on_a_type_alias() {
        for (_, kind, _) in StressKind::ROWS {
            let src = format!("@stress({kind})\ntype T = [3]Int\n");
            assert!(errors(&src, DeclKind::Alias).is_empty(), "{kind}");
        }
        for bad in [
            "@stress(forest)",
            "@stress",
            "@stress(tree, dag)",
            "@stress(kind: tree)",
        ] {
            let got = errors(&format!("{bad}\ntype T = [3]Int\n"), DeclKind::Alias);
            assert_eq!(got.len(), 1, "{bad}");
            assert!(got[0].starts_with("E0325"), "{bad}: {got:?}");
        }
        let on_fn = errors("@stress(tree)\nfn f() = 1\n", DeclKind::Fn { params: 0 });
        assert!(on_fn[0].starts_with("E0325"), "{on_fn:?}");
    }

    #[test]
    fn a_trace_role_fn_takes_one_parameter() {
        let src = "@role(trace)\nfn t(x: Bool) uses io = 1\n";
        assert!(errors(src, DeclKind::Fn { params: 1 }).is_empty());
        for params in [0, 2] {
            let got = errors(src, DeclKind::Fn { params });
            assert_eq!(got.len(), 1, "{params}");
            assert!(got[0].contains("`@role(trace)`"), "{got:?}");
        }
        assert!(errors("@role(print)\nfn p() = 1\n", DeclKind::Fn { params: 0 }).is_empty());
    }
}

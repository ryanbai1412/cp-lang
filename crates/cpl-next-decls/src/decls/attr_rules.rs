//! The rules a decorator follows beyond its name: what each `@name` may sit on and what its
//! arguments are. Collect calls `check` once per decorator, where forms reads it (R11.6).
use crate::attrs::{Attr, DeclKind};
use crate::decls::forms::AttrForm;
use crate::decls::Role;
use crate::diag::Msg;
use cpl_diag::{codes, msgs};
use cpl_next_ir::ids::ItemId;
use cpl_next_syntax::ast::{self, AstNode};

/// The shapes `cpl stress` draws a type alias as (decision 25.32).
pub const STRESS_KINDS: [&str; 6] = ["perm", "tree", "simple", "connected", "dag", "digraph"];

/// The errors of the decorator `attr` on a declaration of kind `on` in `item`.
pub fn check(attr: &AttrForm, on: DeclKind, item: &ItemId, errors: &mut Vec<Msg>) {
    let Some(node) = &attr.node else {
        return;
    };
    let msg = match attr.attr {
        Attr::Stress => stress(node, on),
        Attr::Role => role(node, on),
        _ => None,
    };
    errors.extend(msg.map(|m| Msg::at(item.at(node.syntax().span()), m)));
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
        if kind.is_some_and(|k| STRESS_KINDS.contains(&k.as_str())) {
            return None;
        }
        let kinds = STRESS_KINDS.map(|k| format!("`{k}`")).join(", ");
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

#[cfg(test)]
mod tests {
    use super::*;

    fn errors(src: &str, on: DeclKind) -> Vec<String> {
        let file = cpl_next_syntax::parse(src);
        let node = (file.root.descendants())
            .find_map(ast::Decorator::cast)
            .expect("a decorator");
        let attr = Attr::of(node.name().expect("a name").text()).expect("an attribute");
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
            .map(|m| format!("{}: {}", m.code, m.text))
            .collect()
    }

    #[test]
    fn stress_names_one_kind_on_a_type_alias() {
        for kind in STRESS_KINDS {
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

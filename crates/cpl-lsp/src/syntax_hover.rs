//! What type syntax means, for hover on the tokens no name owns: the `[`,
//! `]` and `..` of `[0..n]Int`, the `?` of `?Int`, the `[n]` of
//! `[n].filled(0)`. Names inside a type (`Int`, `V`) keep the checker's hover.

use cpl_check::Analysis;
use cpl_diag::Span;
use cpl_syntax::ast::*;
use cpl_syntax::visit::{walk_stmt, Visitor};

/// The hover for type syntax at `offset`, when that is what's there.
pub fn hover(analysis: &Analysis, offset: u32) -> Option<cpl_check::Hover> {
    if let Some(hover) = builtin_hover(&analysis.text, offset) {
        return Some(hover);
    }
    let item = analysis.items.iter().rev().find(|i| i.offset <= offset)?;
    let src = analysis.text.get(item.offset as usize..)?;
    let (span, text) = Written::of(&item.syntax.parsed.stmts).explain(src, offset - item.offset)?;
    Some(cpl_check::Hover {
        span: span.shift(item.offset),
        text,
        doc: None,
        cost: None,
    })
}

/// The names the compiler itself defines, which no declaration shows:
/// `spec/core/03-numbers-layout.md` and `spec/core/02-values-types.md`.
const BUILTINS: &[(&str, &str, &str)] = &[
    ("Int", "type Int", "The default integer, `-(2^63 - 1)..=2^63 - 1`, 8 bytes. Write `lo..=hi` for a bounded one: it is stored in the fewest bytes that hold it. Arithmetic not proved in range traps in a checked build."),
    ("I128", "type I128", "A 128-bit whole number, for products past an `Int`."),
    ("U64", "type U64", "A 64-bit unsigned number whose arithmetic wraps, for hashes and bit tricks."),
    ("Bool", "type Bool", "`true` or `false`, 1 byte."),
    ("Char", "type Char", "One ASCII character, `0..=127`, 1 byte."),
    ("Str", "type Str", "A string of `Char`s, owned and growable."),
    ("F64", "type F64", "An IEEE double; `==` is exact and `close(a, b, eps:)` compares with a tolerance."),
    ("List", "type List<T>", "An owned growable sequence of `T`, indexed from 0."),
    ("Map", "type Map<K, V>", "A map from `K` to `V`; `m[k]` costs O(1) expected."),
    ("Set", "type Set<T>", "A set of `T`, each held once."),
    ("None", "None: ?T", "The absent value of an optional `?T`."),
    ("Some", "Some(x): ?T", "The optional `?T` holding `x`."),
];

/// The hover for a name the compiler defines (`Int`, `List`, `None`).
fn builtin_hover(text: &str, offset: u32) -> Option<cpl_check::Hover> {
    let token = cpl_syntax::lex(text).tokens.into_iter().find(|t| {
        t.kind == cpl_syntax::TokenKind::Ident && t.span.start <= offset && offset < t.span.end
    })?;
    let word = text.get(token.span.start as usize..token.span.end as usize)?;
    let (_, shown, doc) = BUILTINS.iter().find(|(name, ..)| *name == word)?;
    Some(cpl_check::Hover {
        span: token.span,
        text: shown.to_string(),
        doc: Some(doc.to_string()),
        cost: None,
    })
}

/// Every type written in `stmts`, outermost only, and every `[e].filled(x)`
/// as (domain, whole) spans.
#[derive(Default)]
pub struct Written {
    pub types: Vec<Type>,
    pub fills: Vec<(Span, Span)>,
}

impl Written {
    pub fn of(stmts: &[Stmt]) -> Written {
        let mut out = Written::default();
        for stmt in stmts {
            walk_stmt(stmt, &mut out);
        }
        cpl_diag::sort_by_key(&mut out.types, |t| t.span.order());
        out.types.dedup_by_key(|t| t.span);
        out
    }

    /// The innermost written type syntax at `at`, explained.
    pub fn explain(&self, src: &str, at: u32) -> Option<(Span, String)> {
        let inside = |s: Span| s.start <= at && at < s.end;
        if let Some(ty) = self.types.iter().find(|t| inside(t.span)) {
            let ty = innermost(ty, at);
            return meaning(ty, src).map(|m| (ty.span, m));
        }
        let (domain, _) = self.fills.iter().find(|(_, w)| inside(*w))?;
        // Only the brackets around the domain, not the names inside it.
        let open = src[..domain.start as usize].rfind('[')? as u32;
        let close = domain.end + src[domain.end as usize..].find(']')? as u32;
        if at != open && at != close {
            return None;
        }
        let shown = &src[domain.start as usize..domain.end as usize];
        let index = if shown.starts_with(|c: char| c.is_ascii_uppercase()) {
            format!("each index of the domain `{shown}`")
        } else {
            format!("each index in `0..{shown}`")
        };
        let span = Span {
            start: open,
            end: close + 1,
        };
        Some((
            span,
            format!("`[{shown}]`: an array with one cell for {index}; `[{shown}]T` is its type"),
        ))
    }
}

impl Visitor for Written {
    fn stmt(&mut self, stmt: &Stmt) {
        match &stmt.kind {
            StmtKind::Let { ty, .. } | StmtKind::Const { ty, .. } => {
                self.types.extend(ty.iter().cloned())
            }
            StmtKind::Fn(f) => self.fn_decl(f),
            StmtKind::Struct(s) => {
                self.types.extend(s.fields.iter().map(|f| f.ty.clone()));
                self.types
                    .extend(s.generic_defaults.iter().flatten().cloned());
                for method in &s.methods {
                    walk_stmt(method, self);
                }
            }
            StmtKind::Enum(e) => {
                for variant in &e.variants {
                    self.types
                        .extend(variant.fields.iter().map(|(_, t)| t.clone()));
                }
                for method in &e.methods {
                    walk_stmt(method, self);
                }
            }
            StmtKind::TypeAlias { ty, .. } => self.types.push(ty.clone()),
            StmtKind::Impl { target, .. } => self.types.push(target.clone()),
            _ => {}
        }
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Generic { args, .. } => self.types.extend(args.iter().cloned()),
            ExprKind::Table { domains, .. } => self.types.extend(domains.iter().cloned()),
            ExprKind::Lambda {
                param_tys, ret_ty, ..
            } => {
                self.types.extend(param_tys.iter().flatten().cloned());
                self.types.extend(ret_ty.iter().cloned());
            }
            ExprKind::TableFill { domain, .. } => self.fills.push((domain.span, expr.span)),
            // `[n].filled(x)`: the list `[n]` read as the domain `0..n`.
            ExprKind::Field { base, name } if name.text == "filled" => {
                if let ExprKind::List(items) = &base.kind {
                    if let [domain] = &items[..] {
                        self.fills.push((domain.span, base.span));
                    }
                }
            }
            _ => {}
        }
    }
}

impl Written {
    fn fn_decl(&mut self, f: &FnDecl) {
        self.types
            .extend(f.params.iter().filter_map(|p| p.ty.clone()));
        self.types.extend(f.ret.iter().cloned());
    }
}

fn children(ty: &Type) -> Vec<&Type> {
    match &ty.kind {
        TypeKind::Named { args, .. } => args.iter().collect(),
        TypeKind::Array { index, elem } => index.iter().chain([&**elem]).collect(),
        TypeKind::OwnDomain { elem, .. } => vec![elem],
        TypeKind::Optional(t) | TypeKind::Mut(t) | TypeKind::Yields(t) => vec![t],
        TypeKind::Tuple(ts) => ts.iter().collect(),
        TypeKind::Record(fs) | TypeKind::Row(fs) => fs.iter().map(|(_, t)| t).collect(),
        TypeKind::RowWithout { prefix, base, .. } => {
            prefix.iter().map(|(_, t)| t).chain([&**base]).collect()
        }
        TypeKind::Fn { params, ret } => params.iter().chain([&**ret]).collect(),
        TypeKind::ConstrainedStr { len, .. } => len.iter().map(|t| &**t).collect(),
        _ => Vec::new(),
    }
}

fn innermost(ty: &Type, at: u32) -> &Type {
    children(ty)
        .into_iter()
        .find(|t| t.span.start <= at && at < t.span.end)
        .map_or(ty, |t| innermost(t, at))
}

fn text(ty: &Type, src: &str) -> String {
    src.get(ty.span.start as usize..ty.span.end as usize)
        .unwrap_or("")
        .to_string()
}

fn meaning(ty: &Type, src: &str) -> Option<String> {
    let shown = text(ty, src);
    let what = match &ty.kind {
        TypeKind::Array { index, elem } => {
            let per: Vec<String> = index.iter().map(|i| index_words(i, src)).collect();
            format!(
                "an array of `{}` with one cell for {}; an index of that type is in bounds by its type",
                text(elem, src),
                per.join(" and ")
            )
        }
        TypeKind::OwnDomain { name, elem } => format!(
            "an array of `{}` that brings its own domain `{}`, the indices it was built with",
            text(elem, src),
            name.text
        ),
        TypeKind::Range { inclusive, .. } => format!(
            "an `Int` known to lie in `{shown}` ({})",
            if *inclusive {
                "both ends included"
            } else {
                "the upper end excluded"
            }
        ),
        TypeKind::Optional(inner) => {
            format!("a value of type `{}`, or `None`", text(inner, src))
        }
        TypeKind::Tuple(_) => "a tuple: its parts by position".to_string(),
        TypeKind::Record(_) => "a record: its parts by name".to_string(),
        TypeKind::Fn { .. } => "a function type: parameter types `->` result".to_string(),
        TypeKind::Mut(inner) => format!("a `{}` the function may change", text(inner, src)),
        TypeKind::Yields(inner) => format!("a generator yielding `{}`", text(inner, src)),
        TypeKind::Named { name, args } if !args.is_empty() => format!(
            "`{}` with type argument{} {}",
            name.text,
            if args.len() == 1 { "" } else { "s" },
            args.iter()
                .map(|a| format!("`{}`", text(a, src)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
        _ => return None,
    };
    Some(format!("`{shown}`: {what}"))
}

fn index_words(index: &Type, src: &str) -> String {
    match &index.kind {
        TypeKind::Range { .. } => format!("each `Int` in `{}`", text(index, src)),
        _ => format!("each index of `{}`", text(index, src)),
    }
}

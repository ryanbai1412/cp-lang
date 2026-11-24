//! `int_vertex_ids` (L0533): a table over `0..=n` indexed by ids read as
//! `1..=n`. Slot 0 is never an id, and the ids stay plain `Int`s, where
//! `domain V = 1..=n` makes the table `[V]T` and every id a checked `V`.

use std::collections::{HashMap, HashSet};

use crate::walk::{unparen, Visitor};
use crate::{File, Finding, INT_VERTEX_IDS};
use cpl_syntax::ast::*;

use super::hand_input::parts;
use super::shapes::squash;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = Ids {
        src: file.src,
        base: 0,
        read_bounds: HashSet::new(),
        indexes: HashMap::new(),
        from_zero: HashSet::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let mut out = Vec::new();
    for (base, stmt) in file.top_level() {
        let StmtKind::Let {
            pat,
            ty,
            init: Some(init),
            ..
        } = &stmt.kind
        else {
            continue;
        };
        let PatternKind::Name(table) = &pat.kind else {
            continue;
        };
        let ty = ty
            .as_ref()
            .map(|t| squash(t.span.shift(base).text(file.src)));
        let init = squash(init.span.shift(base).text(file.src));
        let Some(n) = zero_based_bound(ty.as_deref(), &init) else {
            continue;
        };
        if !v.read_bounds.contains(n) {
            continue;
        }
        let Some(Some(ids)) = v.indexes.get(table.as_str()) else {
            continue;
        };
        if ids.iter().any(|i| v.from_zero.contains(i)) {
            continue;
        }
        let diag = lint_rules::INT_VERTEX_IDS.warning(stmt.span.shift(base), &[&table, &n, &n])
        .with_help(format!(
            "declare the ids as a domain: `domain V = 1..={n}`, read them as `V` and make `{table}` a `[V]T` (`[V] v => v` for a table that starts as each id); every slot is then an id, and an id out of range traps where it is read"
        ));
        out.push(Finding {
            lint: &INT_VERTEX_IDS,
            diag,
        });
    }
    out
}

/// The `n` of a table that spans `0..=n`: `[0..=n]T` or `[i for i in 0..=n]`,
/// as written without whitespace, where `n` is
/// a name: a literal bound (`[0..=90]`) is a value range, not a count of
/// ids. A table of fresh lists (`[[] for _ in 0..=n]`) is an adjacency list,
/// which `manual_adjacency_list` replaces with a `Graph` over `V`.
fn zero_based_bound<'a>(ty: Option<&'a str>, init: &'a str) -> Option<&'a str> {
    let ident = |s: &'a str| {
        (s.starts_with(|c: char| c.is_alphabetic())
            && s.chars().all(|c| c.is_alphanumeric() || c == '_'))
        .then_some(s)
    };
    if let Some(rest) = ty.and_then(|t| t.strip_prefix("[0..=")) {
        return ident(&rest[..rest.find(']')?]);
    }
    let comp = init.strip_prefix('[')?.strip_suffix(']')?;
    if comp.starts_with("[]for") {
        return None;
    }
    ident(&comp[comp.rfind("in0..=")? + "in0..=".len()..])
}

struct Ids<'a> {
    src: &'a str,
    base: u32,
    /// Each `n` some name is read as `1..=n` for.
    read_bounds: HashSet<String>,
    /// Each indexed name and the plain names that index it; `None` once an
    /// index is anything else (`t[0]`, `t[i - 1]`), which reads the table
    /// as positions or cuts, not ids.
    indexes: HashMap<String, Option<Vec<String>>>,
    /// Loop variables that start at 0.
    from_zero: HashSet<String>,
}

impl Ids<'_> {
    fn text(&self, e: &Expr) -> String {
        squash(e.span.shift(self.base).text(self.src))
    }

    fn walked(&mut self, pat: &Pattern, iter: &Expr) {
        if let PatternKind::Name(v) = &pat.kind {
            if self.text(iter).starts_with("0..") {
                self.from_zero.insert(v.to_string());
            }
        }
    }
}

impl Visitor for Ids<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        let StmtKind::Let {
            ty: Some(ty),
            init: Some(init),
            ..
        } = &stmt.kind
        else {
            return;
        };
        if self.text(init) != "read()" {
            return;
        }
        let ty = squash(ty.span.shift(self.base).text(self.src));
        for part in parts(&ty) {
            if let Some(n) = part.strip_prefix("1..=") {
                self.read_bounds.insert(n.to_string());
            }
        }
    }

    fn for_loop(&mut self, pat: &Pattern, iter: &Expr) {
        self.walked(pat, iter);
    }

    fn expr(&mut self, expr: &Expr) {
        match &expr.kind {
            ExprKind::Index { base, indices } => {
                let ExprKind::Name(table) = &unparen(base).kind else {
                    return;
                };
                let id = match &indices[..] {
                    [i] => match &unparen(i).kind {
                        ExprKind::Name(n) => Some(n.to_string()),
                        _ => None,
                    },
                    _ => None,
                };
                let entry = self
                    .indexes
                    .entry(table.to_string())
                    .or_insert_with(|| Some(Vec::new()));
                match (entry.as_mut(), id) {
                    (Some(ids), Some(id)) => ids.push(id),
                    _ => *entry = None,
                }
            }
            ExprKind::Comprehension { clauses, .. } => {
                for c in clauses {
                    self.walked(&c.pat, &c.iter);
                }
            }
            _ => {}
        }
    }
}

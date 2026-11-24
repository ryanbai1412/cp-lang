//! Input read in a shape the program then works around: one list of points
//! read as parallel coordinate arrays (`tuple_points`).

use crate::walk::Visitor;
use crate::{File, Finding, TUPLE_POINTS};
use cpl_diag::Span;
use cpl_syntax::ast::*;

use super::shapes::squash;
use super::Warn;
use cpl_diag::msgs::lint_rules;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandInput {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    let order = crate::code_order(v.out.iter().map(|f| f.diag.code));
    cpl_diag::sort_by_key(&mut v.out, |f| crate::code_rank(&order, f.diag.code));
    cpl_diag::sort_by_key(&mut v.out, |f| f.diag.span.start as u64);
    v.out
        .dedup_by_key(|f| (f.diag.span.start, f.diag.span.end, f.diag.code));
    v.out
}

struct HandInput<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

/// Splits `(a, b)` into its top-level parts; any other text is one part.
pub(super) fn parts(ty: &str) -> Vec<&str> {
    let Some(inner) = ty.strip_prefix('(').and_then(|t| t.strip_suffix(')')) else {
        return vec![ty];
    };
    let (mut out, mut depth, mut start) = (Vec::new(), 0, 0);
    for (i, c) in inner.char_indices() {
        match c {
            '(' | '[' | '<' => depth += 1,
            ')' | ']' | '>' => depth -= 1,
            ',' if depth == 0 => {
                out.push(&inner[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    out.push(&inner[start..]);
    out
}

/// `xs[i] * ys[j]` or `xs[i] - xs[j]`: one coordinate of two different
/// points in one term, the arithmetic of vectors and cross products. Data
/// columns combine at one index (`x[i] * y[i]`).
fn mixes_points(code: &str, x: &str, y: &str) -> bool {
    let index_at = |at: usize, name: &str| -> Option<(&str, usize)> {
        let rest = code.get(at..)?.strip_prefix(name)?.strip_prefix('[')?;
        let end = rest.find(']')?;
        Some((&rest[..end], at + name.len() + end + 2))
    };
    code.match_indices(x).any(|(at, _)| {
        let boundary = !code[..at]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric() || c == '_');
        let Some((i, end)) = index_at(at, x).filter(|_| boundary) else {
            return false;
        };
        let Some(op) = code[end..]
            .chars()
            .next()
            .filter(|c| matches!(c, '-' | '*'))
        else {
            return false;
        };
        [x, y]
            .iter()
            .any(|n| index_at(end + op.len_utf8(), n).is_some_and(|(j, _)| j != i))
    })
}

/// The `y` name that pairs with an `x` name: `xs`/`ys`, `x`/`y`, `px`/`py`.
fn y_of(x: &str) -> Option<String> {
    let i = x.find('x')?;
    (x.len() <= 3).then(|| format!("{}y{}", &x[..i], &x[i + 1..]))
}

impl super::Warn for HandInput<'_> {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32) {
        (&mut self.out, 0)
    }
}

impl HandInput<'_> {
    fn text(&self, base: u32, span: Span) -> &str {
        span.shift(base).text(self.src)
    }

    /// The statements as written, without whitespace.
    fn squashed(&self, stmts: &[(u32, &Stmt)]) -> String {
        let texts: Vec<String> = stmts
            .iter()
            .map(|(b, s)| squash(self.text(*b, s.span)))
            .collect();
        texts.join(";")
    }

    /// `let x: T = read()`: the names and their declared types, as written.
    fn read_let<'s>(&self, (base, stmt): (u32, &'s Stmt)) -> Option<(Vec<&'s str>, String)> {
        let StmtKind::Let {
            pat,
            ty: Some(ty),
            init: Some(init),
            ..
        } = &stmt.kind
        else {
            return None;
        };
        if squash(self.text(base, init.span)) != "read()" {
            return None;
        }
        let names = match &pat.kind {
            PatternKind::Name(n) => vec![n.as_str()],
            PatternKind::Tuple(ps) => ps
                .iter()
                .map(|p| match &p.kind {
                    PatternKind::Name(n) => Some(n.as_str()),
                    _ => None,
                })
                .collect::<Option<_>>()?,
            _ => return None,
        };
        Some((names, squash(self.text(base, ty.span))))
    }

    /// `let xs: [D]Int = read()` next to `let ys: [D]Int = read()`, both
    /// indexed later.
    fn parallel_coords(&mut self, stmts: &[(u32, &Stmt)]) {
        for (k, pair) in stmts.windows(2).enumerate() {
            let (Some((xs, tx)), Some((ys, ty))) = (self.read_let(pair[0]), self.read_let(pair[1]))
            else {
                continue;
            };
            let ([x], [y]) = (xs.as_slice(), ys.as_slice()) else {
                continue;
            };
            if tx != ty || y_of(x).as_deref() != Some(*y) {
                continue;
            }
            let Some(dom) = tx.strip_prefix('[').and_then(|t| t.split(']').next()) else {
                continue;
            };
            let rest = self.squashed(&stmts[k + 2..]);
            if rest.contains(&format!("Pt({x}[")) || !mixes_points(&rest, x, y) {
                continue;
            }
            let (x, y, dom) = (x.to_string(), y.to_string(), dom.to_string());
            self.warn_at(
                &TUPLE_POINTS,
                pair[0].1.span.shift(pair[0].0).to(pair[1].1.span.shift(pair[1].0)),
&lint_rules::TUPLE_POINTS, &[&x, &y],
                format!("build the points once: `let p = [{dom}] i => Pt({x}[i], {y}[i])`; then `p[i] - p[j]`, `p[i].cross(p[j])` and `p[i].dot(p[j])` replace the coordinate arithmetic"),
            );
        }
    }
}

impl Visitor for HandInput<'_> {
    fn stmts(&mut self, stmts: &[(u32, &Stmt)]) {
        let at: Vec<(u32, &Stmt)> = stmts.iter().map(|&(b, s)| (self.base + b, s)).collect();
        self.parallel_coords(&at);
    }
}

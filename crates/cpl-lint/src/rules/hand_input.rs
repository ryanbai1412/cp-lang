//! Input read in a shape the program then works around: positions read
//! 1-based and shifted back at each use (`query_index_shift`), and one list
//! of points read as parallel coordinate arrays (`tuple_points`).

use crate::walk::Visitor;
use crate::{File, Finding, Lint, QUERY_INDEX_SHIFT, TUPLE_POINTS};
use cpl_diag::{Diagnostic, Span};
use cpl_syntax::ast::*;

use super::shapes::squash;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandInput {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    // A file's top-level statements come in several items, each with its own
    // offset, so they are also read as one run.
    let top: Vec<(u32, &Stmt)> = file.top_level().collect();
    v.query_shift(&top);
    v.parallel_coords(&top);
    v.out.sort_by_key(|f| (f.diag.span.start, f.diag.code));
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
fn parts(ty: &str) -> Vec<&str> {
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

/// Whether `[n - 1]` is read somewhere in `code`. A shift that is only
/// written (`d[l - 1] += x` beside `d[r] -= x`) is a difference array, which
/// `diff_by_hand` reports.
fn reads_shifted(code: &str, n: &str) -> bool {
    let shifted = format!("[{n}-1]");
    code.match_indices(&shifted).any(|(i, _)| {
        let after = &code[i + shifted.len()..];
        let written = ["+=", "-=", "*=", "^=", "|=", "&="]
            .iter()
            .any(|op| after.starts_with(op))
            || (after.starts_with('=') && !after.starts_with("=="));
        !written
    })
}

/// Whether `n` bounds a range (`0..n`, `domain V = 1..=n`) or sizes a type
/// (`[n]`, `[n - 1](V, V)`) anywhere in `code`: then it is a count, and `a[n - 1]` is the last element.
fn is_size(code: &str, n: &str) -> bool {
    let bounds = [format!("..{n}"), format!("..={n}")].into_iter().any(|p| {
        code.match_indices(&p).any(|(i, _)| {
            !code[i + p.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_alphanumeric() || c == '_')
        })
    });
    // `[n]Int`, `[n - 1](V, V)`: a type follows, where an index would not.
    let sizes = [format!("[{n}]"), format!("[{n}-1]")].into_iter().any(|p| {
        code.match_indices(&p).any(|(i, _)| {
            code[i + p.len()..]
                .chars()
                .next()
                .is_some_and(|c| c.is_ascii_digit() || c.is_uppercase() || c == '(' || c == '[')
        })
    });
    bounds || sizes
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

    fn warn(&mut self, lint: &'static Lint, span: Span, msg: String, help: String) {
        let diag = Diagnostic::warning(lint.code, span, msg).with_help(help);
        self.out.push(Finding { lint, diag });
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

    /// `let (l, r): (1..=n, 1..=n) = read()`, then `a[l - 1]`.
    fn query_shift(&mut self, stmts: &[(u32, &Stmt)]) {
        for (k, &(base, stmt)) in stmts.iter().enumerate() {
            let Some((names, ty)) = self.read_let((base, stmt)) else {
                continue;
            };
            let tys = parts(&ty);
            if tys.len() != names.len() {
                continue;
            }
            let rest = self.squashed(&stmts[k + 1..]);
            let Some((name, from)) = names.iter().zip(&tys).find(|(n, t)| {
                t.starts_with("1..") && reads_shifted(&rest, n) && !is_size(&squash(self.src), n)
            }) else {
                continue;
            };
            let (name, from) = (name.to_string(), from.to_string());
            self.warn(
                &QUERY_INDEX_SHIFT,
                stmt.span.shift(base),
                format!("`{name}` is read as `{from}` and shifted back with `{name} - 1` to index"),
                format!("declare the array over the positions as they are read: `domain I = {from}`, `let a: [I]Int = read()`, and read `{name}` as `I`; then `a[{name}]` needs no `- 1`, and `a.scan(+).fold(l..=r)` sums a query `l..=r`"),
            );
        }
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
            self.warn(
                &TUPLE_POINTS,
                pair[0].1.span.shift(pair[0].0).to(pair[1].1.span.shift(pair[1].0)),
                format!("`{x}` and `{y}` are the coordinates of one list of points"),
                format!("build the points once: `let p = [{dom}] i => Pt({x}[i], {y}[i])`; then `p[i] - p[j]`, `p[i].cross(p[j])` and `p[i].dot(p[j])` replace the coordinate arithmetic"),
            );
        }
    }
}

impl Visitor for HandInput<'_> {
    fn stmts(&mut self, stmts: &[Stmt]) {
        let at: Vec<(u32, &Stmt)> = stmts.iter().map(|s| (self.base, s)).collect();
        self.query_shift(&at);
        self.parallel_coords(&at);
    }
}

//! Walks newcomers spell by hand where std names them: a factorial table
//! (`fact_tables_by_hand`).

use super::shapes::is_ident;
use crate::walk::Visitor;
use crate::{File, Finding, FACT_TABLES_BY_HAND};
use cpl_diag::msgs::lint_rules;
use cpl_syntax::ast::*;

pub(crate) fn check(file: &File) -> Vec<Finding> {
    let mut v = HandSearch {
        src: file.src,
        base: 0,
        out: Vec::new(),
    };
    file.walk(&mut v, |v, base| v.base = base);
    v.out
}

struct HandSearch<'a> {
    src: &'a str,
    base: u32,
    out: Vec<Finding>,
}

/// `text` without comments and without spaces, except one between two words
/// (`if a[i]`, `for i in 0..n`), so a keyword never fuses with a name.
fn squash_code(text: &str) -> String {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    let mut out = String::new();
    for line in text.lines() {
        let code = line.split("//").next().unwrap_or("");
        for c in code.chars().chain([' ']) {
            if c.is_whitespace() {
                if out.ends_with(word) {
                    out.push(' ');
                }
            } else {
                if out.ends_with(' ') && !(word(c) && out.trim_end().ends_with(word)) {
                    out.pop();
                }
                out.push(c);
            }
        }
    }
    out.trim_end().to_string()
}

impl HandSearch<'_> {
    /// `for i in 1..=n: fact[i] = fact[i - 1] * i % P` (or `.push(..)`, or
    /// `* Z(i)` over a modular type).
    fn fact_table(&mut self, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let [only] = body.stmts.as_slice() else {
            return;
        };
        let it = text(iter.span);
        let Some(n) = it
            .strip_prefix("1..=")
            .or_else(|| it.strip_prefix("1.."))
            .or_else(|| it.strip_prefix("2..="))
        else {
            return;
        };
        if !is_ident(&i) {
            return;
        }
        let t = text(only.span);
        let Some(product) = t
            .split_once('=')
            .and_then(|(x, rhs)| {
                let x = x.strip_suffix(&format!("[{i}]"))?;
                is_ident(x).then_some(())?;
                rhs.strip_prefix(&format!("{x}[{i}-1]*"))
            })
            .or_else(|| {
                let (x, rest) = t.split_once(".push(")?;
                is_ident(x).then_some(())?;
                let rest = rest.strip_suffix(')')?;
                rest.strip_prefix(&format!("{x}[{i}-1]*"))
                    .or_else(|| rest.strip_prefix(&format!("{x}.last!*")))
            })
        else {
            return;
        };
        let (factor, modulus) = match product.split_once('%') {
            Some((f, m)) => (f, Some(m)),
            None => (product, None),
        };
        let ty = if factor == i {
            None
        } else {
            let Some(ty) = factor
                .strip_suffix(&format!("({i})"))
                .filter(|ty| is_ident(ty))
            else {
                return;
            };
            Some(ty)
        };
        let z = match (modulus, ty) {
            (Some(p), None) if p.chars().all(|c| c.is_ascii_digit() || c == '_') => {
                format!("Mod<{p}>")
            }
            (None, Some(ty)) => ty.to_string(),
            _ => return,
        };
        let diag = lint_rules::FACT_TABLES_BY_HAND.warning(only.span.shift(self.base), &[])
        .with_help(format!(
            "write `let comb = Comb<{z}>({n})` (`std.mod`): `comb.fact(i)` is `i!`, `comb.inv_fact(i)` is `1 / i!` and `comb.choose(a, b)` the binomial, with no table or `%` to keep in step"
        ));
        self.out.push(Finding {
            lint: &FACT_TABLES_BY_HAND,
            diag,
        });
    }
}

impl Visitor for HandSearch<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        if let StmtKind::For {
            pat, iter, body, ..
        } = &stmt.kind
        {
            self.fact_table(pat, iter, body);
        }
    }
}

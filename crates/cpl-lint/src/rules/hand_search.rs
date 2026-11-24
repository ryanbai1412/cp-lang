//! Walks newcomers spell by hand where std names them: the submasks of a
//! mask (`submask_loop`), a ternary search over integers
//! (`ternary_search_by_hand`), a palindrome test (`palindrome_by_hand`),
//! every subset by its mask (`subset_masks_by_hand`), a factorial table
//! (`fact_tables_by_hand`) and a sliding window sum (`sliding_sum_by_hand`).

use super::shapes::is_ident;
use crate::walk::Visitor;
use crate::{
    File, Finding, FACT_TABLES_BY_HAND, PALINDROME_BY_HAND, SLIDING_SUM_BY_HAND, SUBMASK_LOOP,
    SUBSET_MASKS_BY_HAND, TERNARY_SEARCH_BY_HAND,
};
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
    /// `span` widened to a `var ok = true` (or `false`) on the line above: the
    /// flag the loop sets is part of the same test.
    fn with_flag(&self, span: cpl_diag::Span) -> cpl_diag::Span {
        let head = &self.src[..span.start as usize];
        let above = head.trim_end_matches([' ', '\t']);
        let above = above.strip_suffix('\n').unwrap_or(above);
        let line_start = above.rfind('\n').map_or(0, |i| i + 1);
        let line = above[line_start..].split("//").next().unwrap_or("").trim();
        let flag =
            line.starts_with("var ") && (line.ends_with("= true") || line.ends_with("= false"));
        if !flag {
            return span;
        }
        let indent = above[line_start..].len() - above[line_start..].trim_start().len();
        cpl_diag::Span::new((line_start + indent) as u32, span.end)
    }

    /// `for mask in 0..(1 << n):` testing `(mask >> i) & 1` to pick `a[i]`.
    fn subset_masks(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let m = text(pat.span);
        let it = text(iter.span);
        let Some(n) = it
            .strip_prefix("0..(1<<")
            .and_then(|r| r.strip_suffix(')'))
            .or_else(|| it.strip_prefix("0..1<<"))
        else {
            return;
        };
        let whole = text(body.span);
        let bit = [format!("({m}>>"), format!("{m}&(1<<"), format!("{m}.bit(")]
            .iter()
            .find_map(|head| {
                let at = whole.find(head.as_str())?;
                let pre = whole[..at].trim_end_matches(['(', ' ']);
                let picks = pre.ends_with("if") || pre.ends_with("and");
                let rest = &whole[at + head.len()..];
                let i = rest
                    .split(|c: char| !(c.is_alphanumeric() || c == '_'))
                    .next()?;
                let unset = rest[i.len()..]
                    .trim_start_matches(".ord")
                    .trim_start_matches([')', '&', '1', ' '])
                    .starts_with("==0");
                (picks && !unset && is_ident(i)).then(|| i.to_string())
            });
        let Some(i) = bit else {
            return;
        };
        let idx = format!("[{i}]");
        let Some(xs) = whole.match_indices(&idx).find_map(|(at, _)| {
            let xs = whole[..at]
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            is_ident(xs).then(|| xs.to_string())
        }) else {
            return;
        };
        if !is_ident(&m) || !is_ident(n) {
            return;
        }
        let diag = lint_rules::SUBSET_MASKS_BY_HAND.warning(stmt.span.shift(self.base), &[&xs])
        .with_help(format!(
            "write `for sub in {xs}.subsequences():` (`std.seq`): the same subsets, `sub.items` the chosen elements, and `sub.prune()` skips every extension of one; keep the mask where it is a DP index"
        ));
        self.out.push(Finding {
            lint: &SUBSET_MASKS_BY_HAND,
            diag,
        });
    }

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

    /// `cur += a[i] - a[i - k]` in a loop over `k..n`.
    fn sliding_sum(&mut self, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let it = text(iter.span);
        let Some((k, _)) = it.split_once("..") else {
            return;
        };
        if !is_ident(&i) || k.is_empty() {
            return;
        }
        for st in &body.stmts {
            let t = text(st.span);
            let Some((cur, rhs)) = t.split_once("+=") else {
                continue;
            };
            let Some((add, sub)) = rhs.split_once('-') else {
                continue;
            };
            let Some(a) = add.strip_suffix(&format!("[{i}]")) else {
                continue;
            };
            if !is_ident(cur) || !is_ident(a) || sub != format!("{a}[{i}-{k}]") {
                continue;
            }
            let diag = lint_rules::SLIDING_SUM_BY_HAND.warning(st.span.shift(self.base), &[&cur, &k, &a])
            .with_help(format!(
                "write `let p = Prefix({a})` (`std.prefix`): `p.fold(j - {k}..j)` is the window ending before `j`, for `j` in `{k}..=n`, with no first window to build and no state between iterations"
            ));
            self.out.push(Finding {
                lint: &SLIDING_SUM_BY_HAND,
                diag,
            });
        }
    }

    /// `for i in 0..n / 2:` comparing `s[i]` with `s[n - 1 - i]`.
    fn palindrome(&mut self, stmt: &Stmt, pat: &Pattern, iter: &Expr, body: &Block) {
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let i = text(pat.span);
        let Some(n) = text(iter.span)
            .strip_prefix("0..")
            .and_then(|r| r.strip_suffix("/2"))
            .map(str::to_string)
        else {
            return;
        };
        let whole = text(body.span);
        let mirror = format!("[{n}-1-{i}]");
        let Some(s) = whole.match_indices(&mirror).find_map(|(at, _)| {
            let before = &whole[..at];
            let s = before
                .rsplit(|c: char| !(c.is_alphanumeric() || c == '_'))
                .next()?;
            let before = &before[..before.len() - s.len()];
            ["!=", "=="]
                .iter()
                .any(|op| before.ends_with(&format!("{s}[{i}]{op}")))
                .then(|| s.to_string())
        }) else {
            return;
        };
        if !is_ident(&i) || !is_ident(&s) {
            return;
        }
        let diag = lint_rules::PALINDROME_BY_HAND.warning(self.with_flag(stmt.span.shift(self.base)), &[&s, &i, &s, &n, &i, &s])
        .with_help(format!(
            "write `is_palindrome({s})`: a `Bool`, with no mirrored index or half-length bound to get wrong"
        ));
        self.out.push(Finding {
            lint: &PALINDROME_BY_HAND,
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
            self.palindrome(stmt, pat, iter, body);
            self.subset_masks(stmt, pat, iter, body);
            self.fact_table(pat, iter, body);
            self.sliding_sum(pat, iter, body);
        }
        let StmtKind::While {
            cond: Cond::Expr(cond),
            body,
            else_body: None,
        } = &stmt.kind
        else {
            return;
        };
        let span = stmt.span.shift(self.base);
        let text = |s: cpl_diag::Span| squash_code(s.shift(self.base).text(self.src));
        let c = text(cond.span);
        let Some(last) = body.stmts.last() else {
            return;
        };
        if let Some(s) = c.strip_suffix(">0").or_else(|| c.strip_suffix("!=0")) {
            let step = text(last.span);
            let mask = step.strip_prefix(&format!("{s}=({s}-1)&"));
            if let Some(m) = mask.filter(|m| is_ident(s) && is_ident(m)) {
                let diag = lint_rules::SUBMASK_LOOP.warning(span, &[&s, &s, &m, &m])
                .with_help(format!(
                    "write `for {s} in submasks({m}):` (`use std.math.{{submasks}}`): it yields every submask of `{m}`, largest first, and `0` last, so skip `0` there if the loop must not see it"
                ));
                self.out.push(Finding {
                    lint: &SUBMASK_LOOP,
                    diag,
                });
            }
            return;
        }
        let whole = text(body.span);
        if whole.matches(")/3").count() >= 2 && body.stmts.len() >= 3 {
            let diag = lint_rules::TERNARY_SEARCH_BY_HAND.warning(span, &[])
            .with_help(
                "write `ternary_min(lo..hi, f)!` or `ternary_max(lo..hi, f)!` (`std.search`): golden-section search, about log n calls of `f`, with no stopping rule or final scan to get wrong",
            );
            self.out.push(Finding {
                lint: &TERNARY_SEARCH_BY_HAND,
                diag,
            });
        }
    }
}

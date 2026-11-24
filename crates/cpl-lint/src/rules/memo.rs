//! `memo_key_space`: a `@memo` fn whose parameters are all domains keeps a
//! table with one entry per combination of them. When the input bounds say
//! how many combinations there are, and that many entries cannot fit in
//! memory, say so before the judge does.

use super::bounds::Bounds;
use crate::facts::Facts;
use crate::walk::Visitor;
use crate::{File, Finding, MEMO_KEY_SPACE};
use cpl_check::Ty;
use cpl_diag::Diagnostic;
use cpl_syntax::ast::{ExprKind, Stmt, StmtKind};

/// The least one memo entry can take: an `Int` result.
const BYTES_PER_ENTRY: u128 = 8;

struct Memo<'a> {
    base: u32,
    facts: Option<&'a Facts>,
    bounds: Bounds,
    memory_mb: u64,
    findings: Vec<Finding>,
}

impl Visitor for Memo<'_> {
    fn stmt(&mut self, stmt: &Stmt) {
        let StmtKind::Fn(decl) = &stmt.kind else {
            return;
        };
        let Some(memo) = stmt.decorators.iter().find(|d| d.name.text == "memo") else {
            return;
        };
        let params: Vec<_> = decl
            .params
            .iter()
            .filter(|p| p.name.text != "self")
            .collect();
        // Only a memo keyed by domains is a dense table. `@memo(hash)`, and
        // a key that is a plain number (`x: 1..=1e18`), store the states
        // that are reached and nothing else.
        let hashed = memo
            .args
            .iter()
            .any(|arg| matches!(&arg.value.kind, ExprKind::Name(name) if name == "hash"));
        let dense = params.iter().all(|p| {
            p.ty.is_some()
                && self.facts.is_some_and(|f| {
                    let at = p.name.span.shift(self.base);
                    f.place_type(at)
                        .is_some_and(|ty| !matches!(ty, Ty::Range { .. }))
                })
        });
        if hashed || !dense {
            return;
        }
        // One parameter of unknown size and nothing can be said.
        let sizes: Option<Vec<u128>> = params
            .iter()
            .map(|p| self.bounds.type_size(p.ty.as_ref()?))
            .collect();
        let Some(sizes) = sizes.filter(|sizes| !sizes.is_empty()) else {
            return;
        };
        let states = sizes
            .iter()
            .try_fold(1u128, |total, size| total.checked_mul(*size))
            .unwrap_or(u128::MAX);
        let budget = u128::from(self.memory_mb) << 20;
        if states.saturating_mul(BYTES_PER_ENTRY) <= budget {
            return;
        }
        let product = sizes
            .iter()
            .zip(&params)
            .map(|(size, param)| format!("{} for `{}`", count(*size), param.name.text))
            .collect::<Vec<_>>()
            .join(" x ");
        let diag = Diagnostic::warning(
            MEMO_KEY_SPACE.code,
            decl.name.span.shift(self.base),
            format!(
                "`@memo` on `{}` can hold {} entries, at least {} of memory; the limit is {} MB",
                decl.name.text,
                count(states),
                bytes(states.saturating_mul(BYTES_PER_ENTRY)),
                self.memory_mb,
            ),
        )
        .with_note(format!("one entry per combination of arguments: {product}"))
        .with_help(
            "narrow a parameter's type to the values it really takes, or fill a table in order and keep only the rows the recurrence reads: the memo holds one entry per combination, so each narrower parameter divides its size",
        );
        self.findings.push(Finding {
            lint: &MEMO_KEY_SPACE,
            diag,
        });
    }
}

/// `200000`, or `4.0e10` once the digits stop helping.
fn count(n: u128) -> String {
    if n < 10_000_000 {
        n.to_string()
    } else {
        let exponent = n.ilog10();
        let mantissa = n as f64 / 10f64.powi(exponent as i32);
        format!("{mantissa:.1}e{exponent}")
    }
}

fn bytes(n: u128) -> String {
    const UNITS: [(&str, u32); 3] = [("TB", 40), ("GB", 30), ("MB", 20)];
    for (unit, shift) in UNITS {
        if n >= 1u128 << shift {
            let value = n as f64 / (1u128 << shift) as f64;
            return if value >= 100_000.0 {
                format!("{value:.1e} {unit}")
            } else {
                format!("{value:.0} {unit}")
            };
        }
    }
    format!("{n} bytes")
}

pub(crate) fn check(file: &File, memory_mb: u64) -> Vec<Finding> {
    let mut memo = Memo {
        base: 0,
        facts: file.facts,
        bounds: Bounds::of(file),
        memory_mb,
        findings: Vec::new(),
    };
    file.walk(&mut memo, |memo, base| memo.base = base);
    memo.findings
}

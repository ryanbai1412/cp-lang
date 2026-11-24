//! How wide a position's C++ integer is. A plain domain holds at most
//! `i32::MAX` positions, so its position is an `int`; a product or sum
//! domain (`(D, D)` keys, an enum with payloads) multiplies its parts'
//! counts and is an `ll` unless a proven bound keeps it under `i32::MAX`.

use cpl_ir::{Expr, ExprKind, Program, Ty};

use crate::ranges::Facts;
use crate::walk;

/// By domain, whether its positions need 64 bits, and its count when it is
/// the same small constant on every run.
pub(crate) fn domain_sizes(program: &Program) -> (Vec<bool>, Vec<Option<u32>>) {
    let facts = Facts::collect(program);
    let fixed = (0..program.domains.len())
        .map(|d| {
            let n = facts.fixed_domain_len(d)?;
            (1..=cpl_ir::FIXED_MAX).contains(&n).then_some(n as u32)
        })
        .collect();
    let wide = (0..program.domains.len())
        .map(|d| {
            let summed = facts.domain_len_expr(d).is_some_and(counts_domains);
            let composite = summed || program.products.get(d).is_some_and(Option::is_some);
            composite && max_len(program, &facts, d, 0).is_none_or(|n| n > i64::from(i32::MAX))
        })
        .collect();
    (wide, fixed)
}

/// Whether a length is made of other domains' counts (`C_len * C_len`).
fn counts_domains(e: &Expr) -> bool {
    let mut found = matches!(e.kind, ExprKind::DomainLen(_));
    walk::children(e, &mut |child| found |= counts_domains(child));
    found
}

/// A static bound on how many positions `d` has.
fn max_len(program: &Program, facts: &Facts, d: usize, depth: u8) -> Option<i64> {
    if depth > 8 {
        return None;
    }
    let Some(Some(parts)) = program.products.get(d) else {
        return facts.domain_max_len(d);
    };
    parts.iter().try_fold(1i64, |n, (_, part)| {
        let k = match part {
            Ty::Pos(p) | Ty::Bound(p) => max_len(program, facts, *p, depth + 1)?,
            Ty::Bool => 2,
            Ty::Enum(e) => program.enums[*e].variants.len() as i64,
            _ => return None,
        };
        n.checked_mul(k)
    })
}

pub(crate) fn is_wide(program: &Program, d: usize) -> bool {
    program.wide_domains.get(d).copied().unwrap_or(false)
}

/// The C++ integer a position of `d` is held in.
pub(crate) fn pos_cpp(program: &Program, d: usize) -> &'static str {
    if is_wide(program, d) {
        "ll"
    } else {
        "int"
    }
}

/// The runtime check that makes an ordinal a position of `cast`'s domain.
pub(crate) fn pos_fn(program: &Program, cast: &Expr) -> &'static str {
    match cast.ty {
        Ty::Pos(d) | Ty::Bound(d) if is_wide(program, d) => "cpl::pos_wide",
        _ => "cpl::pos",
    }
}

/// Each domain's position width and the bound that chose it, and whether
/// an array over it is a `std::array`, for `cpl explain`.
pub(crate) fn explained(program: &Program) -> Vec<String> {
    let facts = Facts::collect(program);
    (0..program.domains.len())
        .map(|d| {
            let name = &program.domains[d];
            let composite = program.products.get(d).is_some_and(Option::is_some)
                || facts.domain_len_expr(d).is_some_and(counts_domains);
            let width = match (is_wide(program, d), composite) {
                (true, _) => "`ll` positions: its count multiplies other counts and no bound keeps it under i32::MAX",
                (false, true) => "`int` positions: a bound keeps its count under i32::MAX",
                (false, false) => "`int` positions: a plain domain holds at most i32::MAX",
            };
            match program.fixed_lens.get(d).copied().flatten() {
                Some(n) => format!("domain {name} -> {width}; {n} on every run, so an array over it is a `std::array`"),
                None => format!("domain {name} -> {width}"),
            }
        })
        .collect()
}

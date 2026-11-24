//! The rules. Each is a function from a parsed file to what it found, in
//! file positions; `lint` applies levels and `with lint` afterwards.

mod adjacency;
mod assign;
mod bag;
mod bounds;
mod domain_first;
mod empty_range;
mod euclid;
mod extreme;
mod fields;
mod floats;
mod found_flag;
mod grid;
mod hand_ds;
mod hand_graph;
mod hand_input;
mod hand_loops;
mod hand_math;
mod hand_search;
mod imports;
mod isqrt;
mod letters;
mod lists;
mod literals;
mod map_lookup;
mod memo;
mod mod_div;
mod mod_reduce;
mod neg_heap;
mod noop_mod;
mod overflow;
mod proven;
mod read_loop;
mod scopes;
mod settled;
mod shapes;
mod spelled;
mod taste;
mod test_count;
mod types;
mod unreachable;
mod vertex_ids;
mod while_pop;
mod wide_shift;
mod zero_div;

use crate::{Config, File, Finding, Lint};
use cpl_diag::msgs::Msg;
use cpl_diag::Span;
use cpl_syntax::ast::{BinOp, Expr, ExprKind, UnOp};
use std::fmt::Display;

pub(crate) use proven::*;

pub(crate) fn run(file: &File, config: &Config) -> Vec<Finding> {
    let mut findings = Vec::new();
    findings.extend(adjacency::check(file));
    findings.extend(bag::check(file));
    findings.extend(neg_heap::check(file));
    findings.extend(extreme::check(file));
    findings.extend(letters::check(file));
    findings.extend(scopes::check(file));
    findings.extend(types::check(file));
    findings.extend(assign::check(file));
    findings.extend(while_pop::check(file));
    findings.extend(while_pop::coalesce(file));
    findings.extend(map_lookup::check(file));
    findings.extend(test_count::check(file));
    findings.extend(euclid::check(file));
    findings.extend(isqrt::check(file));
    findings.extend(found_flag::check(file));
    findings.extend(mod_reduce::check(file));
    findings.extend(grid::check(file));
    findings.extend(floats::check(file));
    findings.extend(imports::check(file));
    findings.extend(fields::check(file));
    findings.extend(memo::check(file, config.memory_mb));
    findings.extend(lists::check(file));
    findings.extend(mod_div::check(file));
    findings.extend(taste::check(file));
    findings.extend(literals::check(file));
    findings.extend(spelled::check(file));
    findings.extend(hand_loops::check(file));
    findings.extend(hand_math::check(file));
    findings.extend(unreachable::check(file));
    findings.extend(zero_div::check(file));
    findings.extend(wide_shift::check(file));
    findings.extend(empty_range::check(file));
    findings.extend(domain_first::check(file));
    findings.extend(overflow::check(file));
    findings.extend(settled::check(file));
    findings.extend(noop_mod::check(file));
    findings.extend(hand_graph::check(file));
    findings.extend(hand_ds::check(file));
    findings.extend(hand_input::check(file));
    findings.extend(vertex_ids::check(file));
    findings.extend(hand_search::check(file));
    findings.extend(read_loop::check(file));
    findings
}

/// `name = value` as a diagnostic's witness when `text` is a plain name, the
/// only kind an input can be pinned to.
pub(crate) fn witness(text: &str, value: i128) -> Vec<(String, i128)> {
    let plain = text.starts_with(|c: char| c.is_alphabetic() || c == '_')
        && text.chars().all(|c| c.is_alphanumeric() || c == '_');
    if plain {
        vec![(text.to_string(), value)]
    } else {
        Vec::new()
    }
}

/// A rule that reports into a `Vec<Finding>`, with spans relative to `base`.
trait Warn {
    fn sink(&mut self) -> (&mut Vec<Finding>, u32);

    fn warn_at(
        &mut self,
        lint: &'static Lint,
        span: Span,
        msg: &'static Msg,
        args: &[&dyn Display],
        help: String,
    ) {
        let (out, base) = self.sink();
        let diag = msg.warning(span.shift(base), args).with_help(help);
        out.push(Finding { lint, diag });
    }
}

/// `-1`, `10**18`, `10**9 + 7`, `1000000000000000000` and their negations.
pub(crate) fn constant(e: &Expr) -> Option<i128> {
    match &e.kind {
        ExprKind::Int(v) => Some(*v),
        ExprKind::Paren(inner) => constant(inner),
        ExprKind::Unary {
            op: UnOp::Neg,
            expr,
        } => constant(expr).map(|v| -v),
        ExprKind::Binary {
            op: BinOp::Pow,
            lhs,
            rhs,
            ..
        } => constant(lhs)?.checked_pow(u32::try_from(constant(rhs)?).ok()?),
        ExprKind::Binary {
            op: BinOp::Add,
            lhs,
            rhs,
            ..
        } => constant(lhs)?.checked_add(constant(rhs)?),
        _ => None,
    }
}

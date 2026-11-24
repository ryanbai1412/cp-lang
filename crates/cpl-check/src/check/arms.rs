//! A `match` arm after one that takes every value never runs (R01.7.3).

use super::*;
use cpl_syntax::ast::{Arm, Pattern, PatternKind};

impl Checker<'_> {
    pub(super) fn require_reachable_arms(&mut self, scrutinee: &Ty, arms: &[Arm]) {
        let Some(at) = arms
            .iter()
            .position(|a| a.guard.is_none() && takes_every_value(&a.pat))
        else {
            return;
        };
        let Some(next) = arms.get(at + 1) else {
            return;
        };
        let catch_all = &arms[at].pat;
        // `incr(a, b) =>`, a pattern the parser could not read, only looks like a binding.
        if self.sink.encloses_poison(catch_all.span) {
            return;
        }
        let message = format!(
            "unreachable arm: `{}` above already matches everything",
            self.quote(catch_all.span)
        );
        let variant = self.variant_spelled_like(scrutinee, catch_all);
        // `less`, `greater`, `equal`: every miscased variant is spelled right at once.
        let mut edits = Vec::new();
        for arm in arms[at..].iter().filter(|a| a.guard.is_none()) {
            if let Some(v) = self.variant_spelled_like(scrutinee, &arm.pat) {
                edits.push((arm.pat.span, v));
            }
        }
        if let Some(diag) = self.error(codes::UNREACHABLE_ARM, next.pat.span, message) {
            match variant {
                Some(v) => {
                    diag.help = Some(format!(
                        "a lowercase name in a pattern binds; did you mean `{v}`?"
                    ));
                    let title = if edits.len() == 1 {
                        format!("write `{v}`")
                    } else {
                        "write the variants".to_string()
                    };
                    diag.fixes.push(cpl_diag::Fix { title, edits });
                }
                None => {
                    diag.help = Some(
                        "put the arm that matches everything last, or remove the arms after it"
                            .to_string(),
                    );
                }
            }
        }
    }

    /// `less` against an `Ordering`: the variant a binding is a miscased spelling of.
    fn variant_spelled_like(&mut self, scrutinee: &Ty, pat: &Pattern) -> Option<String> {
        let PatternKind::Name(name) = &pat.kind else {
            return None;
        };
        let Ty::Named { name: ty, .. } = scrutinee else {
            return None;
        };
        let Some(Def::Enum { variants, .. }) = self.type_def(ty).as_deref().cloned() else {
            return None;
        };
        variants
            .into_iter()
            .map(|(v, _)| v)
            .find(|v| v != name && v.eq_ignore_ascii_case(name))
    }
}

/// `_`, a lowercase binding, or a tuple or record of them: no value fails to match.
fn takes_every_value(pat: &Pattern) -> bool {
    match &pat.kind {
        PatternKind::Wildcard | PatternKind::Domain(_) => true,
        PatternKind::Name(n) => !n.starts_with(|c: char| c.is_ascii_uppercase()),
        PatternKind::Tuple(items) => items.iter().all(takes_every_value),
        PatternKind::Record { fields, .. } => fields.iter().all(|(_, p)| takes_every_value(p)),
        PatternKind::Var(inner) => takes_every_value(inner),
        PatternKind::Or(pats) => pats.iter().any(takes_every_value),
        PatternKind::Variant { .. } | PatternKind::Literal(_) | PatternKind::Error => false,
    }
}

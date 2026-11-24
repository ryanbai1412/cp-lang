//! Where a run recorded again after an edit first parts from the one before.

use super::*;

/// Where `after`, a run recorded again after an edit, first parts from
/// `before`. Steps are compared in order: the n-th change of one run against
/// the n-th of the other, `stdout` included.
pub fn first_change(before: &Recording, after: &Recording) -> Option<Change> {
    first_change_in(before, 0..before.len(), after, 0..after.len())
}

/// [`first_change`] over the steps `old` of `before` and `new` of `after`.
pub fn first_change_in(
    before: &Recording,
    old: std::ops::Range<usize>,
    after: &Recording,
    new: std::ops::Range<usize>,
) -> Option<Change> {
    let label = |run: &Recording, i: usize| run.step(i).map(|s| s.label);
    let last = new
        .end
        .saturating_sub(1)
        .max(new.start)
        .min(after.len().saturating_sub(1));
    let (mut was, mut now) = (before.log.start(old.start), after.log.start(new.start));
    for i in 0..old.len().max(new.len()) {
        let (b, a) = (old.start + i, new.start + i);
        if i >= old.len() || i >= new.len() {
            let step = a.min(last);
            return Some(Change {
                step,
                line: after.line(step).unwrap_or(0),
                variable: None,
                before: label(before, b).filter(|_| i < old.len()),
                after: label(after, a).filter(|_| i < new.len()),
            });
        }
        before.log.advance(&mut was, b);
        after.log.advance(&mut now, a);
        // Until now the two held the same: only what this step changed can differ.
        let ids: Vec<&str> = match i {
            0 => before
                .vars
                .iter()
                .chain(&after.vars)
                .map(|v| v.id.as_str())
                .collect(),
            _ => {
                let (old_vars, old_other) = before.log.touched(b);
                let (new_vars, new_other) = after.log.touched(a);
                let mut ids: Vec<&str> = old_vars
                    .iter()
                    .map(|&v| before.vars[v].id.as_str())
                    .collect();
                ids.extend(new_vars.iter().map(|&v| after.vars[v].id.as_str()));
                if old_other || new_other {
                    ids.push("stdout");
                }
                ids
            }
        };
        let value = |run: &Recording, state: &log::State, id: &str| {
            run.vars
                .iter()
                .position(|v| v.id == id)
                .and_then(|var| run.log.value_in(state, var))
        };
        let differs = ids.into_iter().find_map(|id| {
            let (b, a) = (value(before, &was, id), value(after, &now, id));
            (b != a).then(|| (id.to_string(), b, a))
        });
        let differs = differs.or_else(|| {
            let (was, now) = (before.log.locals_in(&was), after.log.locals_in(&now));
            let (b, a) = (members(&was), members(&now));
            let (b, a): (BTreeMap<_, _>, BTreeMap<_, _>) =
                (b.into_iter().collect(), a.into_iter().collect());
            let id = b
                .keys()
                .chain(a.keys())
                .find(|id| b.get(*id) != a.get(*id))?;
            Some((
                id.to_string(),
                b.get(id).map(|v| v.to_string()),
                a.get(id).map(|v| v.to_string()),
            ))
        });
        if let Some((variable, was, now)) = differs {
            return Some(Change {
                step: a,
                line: after.line(a).unwrap_or(0),
                variable: Some(variable),
                before: was,
                after: now,
            });
        }
    }
    None
}

/// The members of a JSON object the tracer wrote, as written.
pub(super) fn members(object: &str) -> Vec<(&str, &str)> {
    let mut out = Vec::new();
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    let (mut key, mut start) = (None, 1);
    for (i, c) in object.char_indices() {
        if quoted {
            match c {
                _ if escaped => escaped = false,
                '\\' => escaped = true,
                '"' => quoted = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '{' | '[' => depth += 1,
            '}' | ']' => depth -= 1,
            ':' if depth == 1 && key.is_none() => key = Some(object[start..i].trim_matches('"')),
            _ => {}
        }
        if (c == ',' && depth == 1) || (c == '}' && depth == 0) {
            if let Some(name) = key.take() {
                let value_start = object[start..i].find(':').map_or(i, |at| start + at + 1);
                out.push((name, &object[value_start..i]));
            }
            start = i + 1;
        }
    }
    out
}

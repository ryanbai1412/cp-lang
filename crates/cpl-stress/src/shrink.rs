//! Structural shrinking: make a failing case smaller while it keeps
//! failing, without ever leaving the input format.
//!
//! The order is big cuts first: lower each size (`n`, `q`) as far as it
//! goes, dropping the array elements that no longer exist; then delete
//! single elements; then lower every value toward its lower bound. After
//! each change the case is fitted back to the spec, so `1..=n` values
//! stay inside the new `n` and every candidate is a valid input.

use crate::gen::{bind, Case, Value};
use crate::spec::{eval, Env, InputSpec, Shape};

fn fit_value(shape: &Shape, value: &Value, env: &Env) -> Option<Value> {
    match (shape, value) {
        (Shape::Int { lo, hi }, Value::Int(v)) => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            (lo <= hi).then(|| Value::Int((*v).clamp(lo, hi)))
        }
        (Shape::Array { len, elem }, Value::Seq(items)) => {
            let len = usize::try_from(eval(len, env)?.max(0)).ok()?;
            // An array can lose elements, not gain them.
            (len <= items.len()).then_some(())?;
            items[..len]
                .iter()
                .map(|item| fit_value(elem, item, env))
                .collect::<Option<_>>()
                .map(Value::Seq)
        }
        (Shape::Tuple(shapes), Value::Tuple(items)) if shapes.len() == items.len() => shapes
            .iter()
            .zip(items)
            .map(|(shape, item)| fit_value(shape, item, env))
            .collect::<Option<_>>()
            .map(Value::Tuple),
        _ => None,
    }
}

/// `case` made valid again after a change: arrays cut to their length,
/// numbers moved inside their range. `None` when it cannot be.
pub fn fit(spec: &InputSpec, case: &Case) -> Option<Case> {
    let mut env: Env = spec.consts.clone();
    let mut values = Vec::new();
    for (field, value) in spec.fields.iter().zip(&case.values) {
        let fitted = match (&field.shape, value) {
            // Each part of `(n, q)` is bound before the next is fitted.
            (Shape::Tuple(shapes), Value::Tuple(items))
                if field.names.len() == shapes.len() && shapes.len() == items.len() =>
            {
                let mut parts = Vec::new();
                for ((shape, item), name) in shapes.iter().zip(items).zip(&field.names) {
                    let part = fit_value(shape, item, &env)?;
                    bind(std::slice::from_ref(name), &part, &mut env);
                    parts.push(part);
                }
                Value::Tuple(parts)
            }
            (shape, value) => fit_value(shape, value, &env)?,
        };
        bind(&field.names, &fitted, &mut env);
        values.push(fitted);
    }
    (values.len() == spec.fields.len()).then_some(Case { values })
}

/// Where a number sits in a case: the field, then the indexes into it.
type Path = Vec<usize>;

fn at<'a>(case: &'a mut Case, path: &[usize]) -> &'a mut Value {
    let mut value = &mut case.values[path[0]];
    for index in &path[1..] {
        value = match value {
            Value::Seq(items) | Value::Tuple(items) => &mut items[*index],
            Value::Int(_) => unreachable!("a path ends at a number"),
        };
    }
    value
}

/// Every number of a case with the lower bound of its range.
fn numbers(spec: &InputSpec, case: &Case) -> Vec<(Path, i128)> {
    fn walk(shape: &Shape, value: &Value, env: &Env, path: &mut Path, out: &mut Vec<(Path, i128)>) {
        match (shape, value) {
            (Shape::Int { lo, .. }, Value::Int(_)) => {
                if let Some(lo) = eval(lo, env) {
                    out.push((path.clone(), lo));
                }
            }
            (Shape::Array { elem, .. }, Value::Seq(items)) => {
                for (index, item) in items.iter().enumerate() {
                    path.push(index);
                    walk(elem, item, env, path, out);
                    path.pop();
                }
            }
            (Shape::Tuple(shapes), Value::Tuple(items)) => {
                for (index, (shape, item)) in shapes.iter().zip(items).enumerate() {
                    path.push(index);
                    walk(shape, item, env, path, out);
                    path.pop();
                }
            }
            _ => {}
        }
    }
    let mut env: Env = spec.consts.clone();
    let mut out = Vec::new();
    for (index, (field, value)) in spec.fields.iter().zip(&case.values).enumerate() {
        walk(&field.shape, value, &env, &mut vec![index], &mut out);
        bind(&field.names, value, &mut env);
    }
    out
}

/// The paths of the numbers that are sizes: named by the program and used
/// in a later bound.
fn sizes(spec: &InputSpec, case: &Case) -> Vec<(Path, i128)> {
    let all = numbers(spec, case);
    let mut out = Vec::new();
    for (index, field) in spec.fields.iter().enumerate() {
        let is_size = |name: &Option<String>| name.as_ref().is_some_and(|n| spec.sizes.contains(n));
        match (&field.shape, field.names.as_slice()) {
            (Shape::Int { .. }, [name]) if is_size(name) => out.push(vec![index]),
            (Shape::Tuple(parts), names) if names.len() == parts.len() => {
                for (part, name) in names.iter().enumerate() {
                    if is_size(name) && matches!(parts[part], Shape::Int { .. }) {
                        out.push(vec![index, part]);
                    }
                }
            }
            _ => {}
        }
    }
    out.into_iter()
        .filter_map(|path| all.iter().find(|(p, _)| *p == path).cloned())
        .collect()
}

/// The lengths of the top-level arrays, by field.
fn lengths(case: &Case) -> Vec<Option<usize>> {
    case.values
        .iter()
        .map(|value| match value {
            Value::Seq(items) => Some(items.len()),
            _ => None,
        })
        .collect()
}

/// Shrinks `case`, which must fail, to a smaller case that still does.
/// `still_fails` runs a candidate; it is called at most `budget` times.
pub fn shrink(
    spec: &InputSpec,
    case: &Case,
    budget: usize,
    still_fails: &mut dyn FnMut(&Case) -> bool,
) -> Case {
    let mut best = case.clone();
    let mut left = budget;
    // Tries a candidate; keeps it when it is valid, smaller and failing.
    let mut attempt = |best: &mut Case, candidate: Case| -> bool {
        let Some(candidate) = fit(spec, &candidate) else {
            return false;
        };
        if candidate.weight() >= best.weight() || left == 0 {
            return false;
        }
        left -= 1;
        if still_fails(&candidate) {
            *best = candidate;
            true
        } else {
            false
        }
    };
    // Lowers the number at `path` as far as the failure allows.
    let lower = |best: &mut Case,
                 path: &Path,
                 lo: i128,
                 attempt: &mut dyn FnMut(&mut Case, Case) -> bool|
     -> bool {
        let Value::Int(start) = *at(best, path) else {
            return false;
        };
        let (mut floor, mut current) = (lo, start);
        while floor < current {
            let target = floor + (current - floor) / 2;
            let mut candidate = best.clone();
            *at(&mut candidate, path) = Value::Int(target);
            if attempt(best, candidate) {
                current = target;
            } else {
                floor = target + 1;
            }
        }
        current < start
    };

    loop {
        let mut improved = false;
        // 1. Sizes: halve, then step.
        for (path, lo) in sizes(spec, &best) {
            improved |= lower(&mut best, &path, lo, &mut attempt);
        }
        // 2. Single elements: lower a size by one and drop one element
        //    from each array that gets shorter, at every position.
        for (path, lo) in sizes(spec, &best) {
            let mut position = 0;
            while let Value::Int(current) = *at(&mut best, &path) {
                if current <= lo {
                    break;
                }
                let mut smaller = best.clone();
                *at(&mut smaller, &path) = Value::Int(current - 1);
                // Which arrays the smaller size shortens.
                let before = lengths(&best);
                let after = fit(spec, &smaller).map(|fitted| lengths(&fitted));
                let Some(after) = after else { break };
                let mut any = false;
                for (field, (old, new)) in before.iter().zip(&after).enumerate() {
                    if let (Some(old), Some(new), Value::Seq(items)) =
                        (old, new, &mut smaller.values[field])
                    {
                        if new + 1 == *old && position < *old {
                            items.remove(position);
                            any = true;
                        }
                    }
                }
                if !any {
                    break;
                }
                if attempt(&mut best, smaller) {
                    improved = true;
                } else {
                    position += 1;
                }
            }
        }
        // 3. Values, toward their lower bound.
        for (path, lo) in numbers(spec, &best) {
            // An earlier step may have removed this number.
            let exists = numbers(spec, &best).iter().any(|(p, _)| *p == path);
            if exists {
                improved |= lower(&mut best, &path, lo, &mut attempt);
            }
        }
        if !improved {
            break;
        }
    }
    best
}

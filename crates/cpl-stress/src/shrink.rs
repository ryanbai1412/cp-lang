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
        (Shape::Char { lo, hi }, Value::Char(c)) => {
            Some(Value::Char((*c).clamp(i128::from(*lo), i128::from(*hi))))
        }
        (Shape::Bool, Value::Int(v)) => Some(Value::Int((*v).clamp(0, 1))),
        // A word keeps at least one letter.
        (Shape::Str, Value::Seq(items)) if !items.is_empty() => Some(Value::Seq(
            items
                .iter()
                .map(|item| Value::Char(item.number().unwrap_or(97).clamp(97, 122)))
                .collect(),
        )),
        // The first `len` values, in the order they were in, renamed
        // to `lo..lo + len`: still a permutation.
        (Shape::Perm { lo, len }, Value::Seq(items)) => {
            let (lo, len) = (eval(lo, env)?, usize::try_from(eval(len, env)?.max(0)).ok()?);
            (len <= items.len()).then_some(())?;
            let kept: Vec<i128> = items[..len].iter().filter_map(Value::number).collect();
            let mut order: Vec<usize> = (0..kept.len()).collect();
            order.sort_by_key(|i| (kept[*i], *i));
            let mut renamed = vec![0; kept.len()];
            for (rank, at) in order.into_iter().enumerate() {
                renamed[at] = lo + rank as i128;
            }
            Some(Value::Seq(renamed.into_iter().map(Value::Int).collect()))
        }
        // The edges that still join two parts of `lo..=hi`, then one
        // from the first vertex to each part left over: still a tree. An
        // edge keeps its weight; a new one takes the first edge's.
        (Shape::Tree { lo, hi, extra }, Value::Seq(items)) => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            let n = usize::try_from((hi - lo + 1).max(0)).ok()?;
            (n <= items.len() + 1).then_some(())?;
            let mut root: Vec<usize> = (0..n).collect();
            fn find(root: &mut [usize], mut v: usize) -> usize {
                while root[v] != v {
                    root[v] = root[root[v]];
                    v = root[v];
                }
                v
            }
            let weight = |ends: &[Value]| -> Option<Vec<Value>> {
                extra
                    .iter()
                    .zip(ends.get(2..)?)
                    .map(|(shape, value)| fit_value(shape, value, env))
                    .collect()
            };
            let first = match items.first() {
                Some(Value::Tuple(ends)) => weight(ends)?,
                _ => Vec::new(),
            };
            let mut edges = Vec::new();
            for item in items {
                let Value::Tuple(ends) = item else { return None };
                let (a, b) = (ends.first()?.number()? - lo, ends.get(1)?.number()? - lo);
                let inside = |v: i128| usize::try_from(v).ok().filter(|v| *v < n);
                let (Some(a), Some(b)) = (inside(a), inside(b)) else {
                    continue;
                };
                let (ra, rb) = (find(&mut root, a), find(&mut root, b));
                if ra != rb {
                    root[ra] = rb;
                    edges.push((a, b, weight(ends)?));
                }
            }
            for v in 1..n {
                let (r0, rv) = (find(&mut root, 0), find(&mut root, v));
                if r0 != rv {
                    root[rv] = r0;
                    edges.push((0, v, first.clone()));
                }
            }
            (edges.iter().all(|edge| edge.2.len() == extra.len())).then_some(())?;
            Some(Value::Seq(
                edges
                    .into_iter()
                    .map(|(a, b, w)| {
                        let mut edge = vec![Value::Int(a as i128 + lo), Value::Int(b as i128 + lo)];
                        edge.extend(w);
                        Value::Tuple(edge)
                    })
                    .collect(),
            ))
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
    fit_in(spec, &spec.consts, case)
}

fn fit_in(spec: &InputSpec, outer: &Env, case: &Case) -> Option<Case> {
    let mut env: Env = outer.clone();
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
    (values.len() == spec.fields.len()).then_some(())?;
    let cases = match &spec.cases {
        Some(inner) => case
            .cases
            .iter()
            .map(|case| fit_in(inner, &env, case))
            .collect::<Option<_>>()?,
        None => Vec::new(),
    };
    Some(Case {
        values,
        cases,
        until_eof: case.until_eof,
    })
}

/// How far a case is from the plainest one of its size: permutations
/// out of order and tree edges written high end first. It breaks ties
/// between cases of one weight, so a permutation shrinks toward
/// `1 2 3 ..` and an edge toward `1 2`.
fn disorder(spec: &InputSpec, case: &Case) -> u128 {
    let mut total = 0u128;
    for (field, value) in spec.fields.iter().zip(&case.values) {
        if let (Shape::Tree { .. }, Value::Seq(edges)) = (&field.shape, value) {
            total += edges
                .iter()
                .filter(|edge| matches!(edge, Value::Tuple(ends) if ends[0].number() > ends[1].number()))
                .count() as u128;
        }
        if let (Shape::Perm { .. }, Value::Seq(items)) = (&field.shape, value) {
            let values: Vec<i128> = items.iter().filter_map(Value::number).collect();
            for (i, a) in values.iter().enumerate() {
                total += values[i + 1..].iter().filter(|b| a > b).count() as u128;
            }
        }
    }
    total
}

/// Where a number sits in a case: the field, then the indexes into it.
type Path = Vec<usize>;

fn at<'a>(case: &'a mut Case, path: &[usize]) -> &'a mut Value {
    let mut value = &mut case.values[path[0]];
    for index in &path[1..] {
        value = match value {
            Value::Seq(items) | Value::Tuple(items) => &mut items[*index],
            Value::Int(_) | Value::Char(_) => unreachable!("a path ends at a number"),
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
            (Shape::Char { lo, .. }, Value::Char(_)) => out.push((path.clone(), i128::from(*lo))),
            (Shape::Bool, Value::Int(_)) => out.push((path.clone(), 0)),
            (Shape::Str, Value::Seq(items)) => {
                for index in 0..items.len() {
                    path.push(index);
                    out.push((path.clone(), 97));
                    path.pop();
                }
            }
            // Lowering one value of a permutation or one end of an edge
            // gives another, plainer one once it is fitted again.
            (Shape::Perm { lo, .. }, Value::Seq(items)) => {
                if let Some(lo) = eval(lo, env) {
                    for index in 0..items.len() {
                        path.push(index);
                        out.push((path.clone(), lo));
                        path.pop();
                    }
                }
            }
            (Shape::Tree { lo, extra, .. }, Value::Seq(items)) => {
                if let Some(lo) = eval(lo, env) {
                    for (index, item) in items.iter().enumerate() {
                        for end in 0..2 {
                            path.extend([index, end]);
                            out.push((path.clone(), lo));
                            path.truncate(path.len() - 2);
                        }
                        let Value::Tuple(ends) = item else { continue };
                        for (at, (shape, value)) in extra.iter().zip(ends.iter().skip(2)).enumerate() {
                            path.extend([index, 2 + at]);
                            walk(shape, value, env, path, out);
                            path.truncate(path.len() - 2);
                        }
                    }
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
/// Removes element `position` from each array of `target` (a copy of
/// `old`) that `new`, `old` fitted to a smaller size, has one fewer of.
fn drop_where_shorter(old: &Value, new: &Value, target: &mut Value, position: usize) -> bool {
    match (old, new, target) {
        (Value::Seq(old), Value::Seq(new), Value::Seq(target)) if new.len() + 1 == old.len() => {
            let fits = position < old.len();
            if fits {
                target.remove(position);
            }
            fits
        }
        (Value::Seq(old), Value::Seq(new), Value::Seq(target))
        | (Value::Tuple(old), Value::Tuple(new), Value::Tuple(target))
            if new.len() == old.len() =>
        {
            let mut any = false;
            for ((old, new), target) in old.iter().zip(new).zip(target.iter_mut()) {
                any |= drop_where_shorter(old, new, target, position);
            }
            any
        }
        _ => false,
    }
}

/// Shrinks `case`, which must fail, to a smaller case that still does.
/// `still_fails` runs a candidate; it is called at most `budget` times.
pub fn shrink(
    spec: &InputSpec,
    case: &Case,
    budget: usize,
    still_fails: &mut dyn FnMut(&Case) -> bool,
) -> Case {
    if let Some(inner) = &spec.cases {
        return shrink_tests(spec, inner, case, budget, still_fails);
    }
    let mut best = case.clone();
    let mut left = budget;
    // Tries a candidate; keeps it when it is valid, smaller and failing.
    let mut attempt = |best: &mut Case, candidate: Case| -> bool {
        let Some(candidate) = fit(spec, &candidate) else {
            return false;
        };
        let measure = |case: &Case| (case.weight(), disorder(spec, case));
        if measure(&candidate) >= measure(best) || left == 0 {
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
        let Some(start) = at(best, path).number() else {
            return false;
        };
        let (mut floor, mut current) = (lo, start);
        while floor < current {
            let target = floor + (current - floor) / 2;
            let mut candidate = best.clone();
            let slot = at(&mut candidate, path);
            *slot = slot.with(target);
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
                // Which arrays the smaller size shortens, at any depth: a
                // smaller `k` shortens every permutation in a list of them.
                let Some(fitted) = fit(spec, &smaller) else { break };
                let mut any = false;
                for (field, new) in fitted.values.iter().enumerate() {
                    any |= drop_where_shorter(&best.values[field], new, &mut smaller.values[field], position);
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
        // 2b. Words: drop one letter at a time.
        for field in 0..best.values.len() {
            if spec.fields[field].shape != Shape::Str {
                continue;
            }
            let mut position = 0;
            while let Value::Seq(letters) = &best.values[field] {
                if letters.len() <= 1 || position >= letters.len() {
                    break;
                }
                let mut smaller = best.clone();
                if let Value::Seq(letters) = &mut smaller.values[field] {
                    letters.remove(position);
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

/// Shrinks a multitest input: first to the one test case that fails on
/// its own, which is then shrunk like any single input. When no test
/// case fails alone (the bug needs what an earlier one left behind),
/// test cases are dropped one at a time and each one left is shrunk with
/// the others held fixed, until neither makes the input smaller.
fn shrink_tests(
    spec: &InputSpec,
    inner: &InputSpec,
    case: &Case,
    budget: usize,
    still_fails: &mut dyn FnMut(&Case) -> bool,
) -> Case {
    let mut left = budget;
    let with = |cases: Vec<Case>| Case {
        values: case.values.clone(),
        cases,
        until_eof: case.until_eof,
    };
    // What a test case sees of the lines above the divider.
    let mut scoped = inner.clone();
    let mut env: Env = spec.consts.clone();
    for (field, value) in spec.fields.iter().zip(&case.values) {
        bind(&field.names, value, &mut env);
    }
    scoped.consts = env;
    if case.cases.len() == 1 {
        let smallest = shrink(&scoped, &case.cases[0], left, &mut |candidate| {
            still_fails(&with(vec![candidate.clone()]))
        });
        return with(vec![smallest]);
    }
    for single in &case.cases {
        if left == 0 {
            break;
        }
        left -= 1;
        let alone = with(vec![single.clone()]);
        if still_fails(&alone) {
            return shrink_tests(spec, inner, &alone, left, still_fails);
        }
    }
    let mut best = case.clone();
    loop {
        let before = best.weight();
        let mut position = 0;
        while position < best.cases.len() && best.cases.len() > 1 && left > 0 {
            left -= 1;
            let mut fewer = best.clone();
            fewer.cases.remove(position);
            if still_fails(&fewer) {
                best = fewer;
            } else {
                position += 1;
            }
        }
        // Each remaining test case shrinks with the others held fixed.
        for i in 0..best.cases.len() {
            if left == 0 {
                break;
            }
            let mut tries = 0;
            let shrunk = shrink(&scoped, &best.cases[i], left, &mut |candidate| {
                tries += 1;
                let mut cases = best.cases.clone();
                cases[i] = candidate.clone();
                still_fails(&with(cases))
            });
            left = left.saturating_sub(tries);
            best.cases[i] = shrunk;
        }
        if best.weight() >= before || left == 0 {
            return best;
        }
    }
}

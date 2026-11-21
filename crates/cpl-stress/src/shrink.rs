//! Structural shrinking: make a failing case smaller while it keeps
//! failing, without ever leaving the input format.
//!
//! The order is big cuts first: lower each size (`n`, `q`) as far as it
//! goes, dropping the array elements that no longer exist; then delete
//! single elements; then lower every value toward its lower bound. After
//! each change the case is fitted back to the spec, so `1..=n` values
//! stay inside the new `n` and every candidate is a valid input.

use std::borrow::Cow;

use crate::gen::{bind, build_parts, Case, Lowest, Value};
use crate::spec::{eval, names_in, Arm, Env, GraphKind, InputSpec, Shape};

/// The representative of `v`'s part in a union-find forest.
fn find(root: &mut [usize], mut v: usize) -> usize {
    while root[v] != v {
        root[v] = root[root[v]];
        v = root[v];
    }
    v
}

fn fit_parts(
    parts: &[Shape],
    binds: &[Vec<Option<String>>],
    items: &[Value],
    env: &Env,
) -> Option<Vec<Value>> {
    let mut env = Cow::Borrowed(env);
    let mut fitted = Vec::new();
    for (at, (shape, item)) in parts.iter().zip(items).enumerate() {
        let part = fit_value(shape, item, &env)?;
        if let Some(names) = binds.get(at).filter(|names| !names.is_empty()) {
            bind(names, &part, env.to_mut());
        }
        fitted.push(part);
    }
    Some(fitted)
}

/// The arm a choice's value read, with its parts after the tag.
fn chosen<'a>(arms: &'a [Arm], items: &'a [Value]) -> Option<(&'a Arm, &'a [Value])> {
    let arm = Arm::taken(arms, items.first()?.number()?)?;
    (items.len() == 1 + arm.parts.len()).then(|| (arm, &items[1..]))
}

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
        (Shape::Maybe { none, .. }, Value::Int(v)) if v == none => Some(Value::Int(*none)),
        (Shape::Maybe { inner, .. }, value) => fit_value(inner, value, env),
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
            let (lo, len) = (
                eval(lo, env)?,
                usize::try_from(eval(len, env)?.max(0)).ok()?,
            );
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
                let Value::Tuple(ends) = item else {
                    return None;
                };
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
        // The first `len` edges, with their ends moved inside `lo..=hi`;
        // `None` when that makes a loop, an edge twice, a cycle in a DAG or
        // a connected graph that is not.
        (
            Shape::Graph {
                lo,
                hi,
                len,
                extra,
                kind,
            },
            Value::Seq(items),
        ) => {
            let (lo, hi) = (eval(lo, env)?, eval(hi, env)?);
            let len = usize::try_from(eval(len, env)?.max(0)).ok()?;
            (lo <= hi && len <= items.len()).then_some(())?;
            let n = usize::try_from(hi - lo + 1).ok()?;
            let mut seen = std::collections::HashSet::new();
            let mut root: Vec<usize> = (0..n).collect();
            let mut into = vec![0usize; n];
            let mut out: Vec<Vec<usize>> = vec![Vec::new(); n];
            let mut parts = n;
            let mut edges = Vec::new();
            for item in &items[..len] {
                let Value::Tuple(ends) = item else {
                    return None;
                };
                let (a, b) = (
                    ends.first()?.number()?.clamp(lo, hi),
                    ends.get(1)?.number()?.clamp(lo, hi),
                );
                let (x, y) = ((a - lo) as usize, (b - lo) as usize);
                let key = if kind.directed() {
                    (x, y)
                } else {
                    (x.min(y), x.max(y))
                };
                (x != y && seen.insert(key)).then_some(())?;
                let (rx, ry) = (find(&mut root, x), find(&mut root, y));
                if rx != ry {
                    root[rx] = ry;
                    parts -= 1;
                }
                out[x].push(y);
                into[y] += 1;
                let mut edge = vec![Value::Int(a), Value::Int(b)];
                for (shape, value) in extra.iter().zip(ends.get(2..)?) {
                    edge.push(fit_value(shape, value, env)?);
                }
                (edge.len() == 2 + extra.len()).then_some(())?;
                edges.push(Value::Tuple(edge));
            }
            match kind {
                GraphKind::Simple | GraphKind::Digraph => {}
                GraphKind::Connected => (parts <= 1).then_some(())?,
                GraphKind::Dag => {
                    let mut ready: Vec<usize> = (0..n).filter(|v| into[*v] == 0).collect();
                    let mut done = 0;
                    while let Some(v) = ready.pop() {
                        done += 1;
                        for &w in &out[v] {
                            into[w] -= 1;
                            if into[w] == 0 {
                                ready.push(w);
                            }
                        }
                    }
                    (done == n).then_some(())?;
                }
            }
            Some(Value::Seq(edges))
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
        (Shape::Tuple { parts, binds }, Value::Tuple(items)) if parts.len() == items.len() => {
            fit_parts(parts, binds, items, env).map(Value::Tuple)
        }
        // A tag that now takes another arm reads that arm's values, each
        // at its low end.
        (Shape::Choice { tag, name, arms }, Value::Tuple(items)) if !items.is_empty() => {
            let literal = |v: i128| {
                arms.iter()
                    .any(|arm| arm.tags.as_ref().is_some_and(|tags| tags.contains(&v)))
            };
            let first = match items[0].number() {
                Some(v) if literal(v) => items[0].clone(),
                _ => fit_value(tag, &items[0], env)?,
            };
            let value = first.number()?;
            let mut env = env.clone();
            env.insert(name.clone(), value);
            let mut fitted = vec![first];
            if let Some(arm) = Arm::taken(arms, value) {
                let rest = &items[1..];
                let parts = (rest.len() == arm.parts.len())
                    .then(|| fit_parts(&arm.parts, &arm.binds, rest, &env))
                    .flatten()
                    .or_else(|| {
                        build_parts(
                            &arm.parts,
                            &arm.binds,
                            &env,
                            &std::collections::HashSet::new(),
                            &mut Lowest,
                        )
                    })?;
                fitted.extend(parts);
            }
            Some(Value::Tuple(fitted))
        }
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
            (Shape::Tuple { parts: shapes, .. }, Value::Tuple(items))
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
            // Values read before it may have changed: draw it again.
            (Shape::Gen { program, .. }, Value::Drawn { seed, .. }) => {
                crate::gen::draw(program, &values, *seed)?
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
/// out of order and undirected edges written high end first. It breaks ties
/// between cases of one weight, so a permutation shrinks toward
/// `1 2 3 ..` and an edge toward `1 2`.
fn disorder(spec: &InputSpec, case: &Case) -> u128 {
    let mut total = 0u128;
    for (field, value) in spec.fields.iter().zip(&case.values) {
        let undirected = match &field.shape {
            Shape::Tree { .. } => true,
            Shape::Graph { kind, .. } => !kind.directed(),
            _ => false,
        };
        if let (true, Value::Seq(edges)) = (undirected, value) {
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
            Value::Int(_) | Value::Char(_) | Value::Drawn { .. } => {
                unreachable!("a path ends at a number")
            }
        };
    }
    value
}

/// The number at `path` in `case`, if there is still one there.
fn number_at(case: &Case, path: &[usize]) -> Option<i128> {
    let (first, rest) = path.split_first()?;
    let mut value = case.values.get(*first)?;
    for index in rest {
        value = match value {
            Value::Seq(items) | Value::Tuple(items) => items.get(*index)?,
            Value::Int(_) | Value::Char(_) | Value::Drawn { .. } => return None,
        };
    }
    value.number()
}

/// Lowers every number of `cases` equal to `value` as one, as far as
/// `accept` takes them; `accept` gives back the cases it keeps. Two
/// values that fail only while equal stop each other when lowered one
/// at a time. Sizes stay out: they have their own steps, and a size
/// equal to a value by chance would hold the value up, and so would a
/// number already at its lower bound.
fn lower_equal(
    spec: &InputSpec,
    cases: &mut Vec<Case>,
    value: i128,
    accept: &mut dyn FnMut(Vec<Case>) -> Option<Vec<Case>>,
) -> bool {
    let holding = |cases: &[Case], x: i128| {
        let mut paths = Vec::new();
        let mut floor = i128::MIN;
        for (index, case) in cases.iter().enumerate() {
            let sizes: Vec<Path> = sizes(spec, case)
                .into_iter()
                .map(|(path, _)| path)
                .collect();
            for (path, lo) in numbers(spec, case) {
                if number_at(case, &path) == Some(x) && lo < x && !sizes.contains(&path) {
                    paths.push((index, path));
                    floor = floor.max(lo);
                }
            }
        }
        (paths, floor)
    };
    let (mut paths, mut floor) = holding(cases, value);
    let mut current = value;
    while paths.len() >= 2 && floor < current {
        let target = floor + (current - floor) / 2;
        let mut candidate = cases.clone();
        for (index, path) in &paths {
            let slot = at(&mut candidate[*index], path);
            *slot = slot.with(target);
        }
        match accept(candidate) {
            Some(kept) => {
                *cases = kept;
                current = target;
                let (now, least) = holding(cases, current);
                paths = now;
                floor = floor.max(least);
            }
            None => floor = target + 1,
        }
    }
    current < value
}

/// The distinct numbers of `cases`, largest first.
fn distinct_numbers(spec: &InputSpec, cases: &[Case]) -> Vec<i128> {
    let mut all: Vec<i128> = cases
        .iter()
        .flat_map(|case| {
            numbers(spec, case)
                .into_iter()
                .filter_map(|(path, _)| number_at(case, &path))
        })
        .collect();
    all.sort_unstable_by(|a, b| b.cmp(a));
    all.dedup();
    all
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
            // Lowered past its type's low end, a value becomes `none`.
            (Shape::Maybe { inner, none }, Value::Int(v)) if v != none => {
                if let Shape::Int { lo, .. } = &**inner {
                    if let Some(lo) = eval(lo, env) {
                        out.push((path.clone(), lo.min(*none)));
                    }
                }
            }
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
            (Shape::Tree { lo, extra, .. } | Shape::Graph { lo, extra, .. }, Value::Seq(items)) => {
                if let Some(lo) = eval(lo, env) {
                    for (index, item) in items.iter().enumerate() {
                        for end in 0..2 {
                            path.extend([index, end]);
                            out.push((path.clone(), lo));
                            path.truncate(path.len() - 2);
                        }
                        let Value::Tuple(ends) = item else { continue };
                        for (at, (shape, value)) in
                            extra.iter().zip(ends.iter().skip(2)).enumerate()
                        {
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
            (Shape::Tuple { parts, binds }, Value::Tuple(items)) => {
                let mut env = Cow::Borrowed(env);
                for (index, (shape, item)) in parts.iter().zip(items).enumerate() {
                    path.push(index);
                    walk(shape, item, &env, path, out);
                    path.pop();
                    if let Some(names) = binds.get(index).filter(|names| !names.is_empty()) {
                        bind(names, item, env.to_mut());
                    }
                }
            }
            (Shape::Choice { tag, name, arms }, Value::Tuple(items)) => {
                path.push(0);
                walk(tag, &items[0], env, path, out);
                path.pop();
                if let Some((arm, rest)) = chosen(arms, items) {
                    let mut env = env.clone();
                    env.insert(name.clone(), items[0].number().unwrap_or(0));
                    for (index, (shape, item)) in arm.parts.iter().zip(rest).enumerate() {
                        path.push(1 + index);
                        walk(shape, item, &env, path, out);
                        path.pop();
                        if let Some(names) = arm.binds.get(index).filter(|names| !names.is_empty())
                        {
                            bind(names, item, &mut env);
                        }
                    }
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

/// The paths of the tags of choices, each with a tag for every arm: a
/// row may read less down another arm than lowering its tag allows.
fn tags(spec: &InputSpec, case: &Case) -> Vec<(Path, Vec<i128>)> {
    fn walk(
        shape: &Shape,
        value: &Value,
        env: &Env,
        path: &mut Path,
        out: &mut Vec<(Path, Vec<i128>)>,
    ) {
        match (shape, value) {
            (Shape::Array { elem, .. }, Value::Seq(items)) => {
                for (index, item) in items.iter().enumerate() {
                    path.push(index);
                    walk(elem, item, env, path, out);
                    path.pop();
                }
            }
            (Shape::Tuple { parts, binds }, Value::Tuple(items)) => {
                let mut env = Cow::Borrowed(env);
                for (index, (shape, item)) in parts.iter().zip(items).enumerate() {
                    path.push(index);
                    walk(shape, item, &env, path, out);
                    path.pop();
                    if let Some(names) = binds.get(index).filter(|names| !names.is_empty()) {
                        bind(names, item, env.to_mut());
                    }
                }
            }
            (Shape::Choice { tag, name, arms }, Value::Tuple(items)) => {
                let named: Vec<i128> = arms
                    .iter()
                    .flat_map(|arm| arm.tags.iter().flatten().copied())
                    .collect();
                let rest = match &**tag {
                    Shape::Int { lo, hi } => eval(lo, env).zip(eval(hi, env)),
                    Shape::Char { lo, hi } => Some((i128::from(*lo), i128::from(*hi))),
                    Shape::Bool => Some((0, 1)),
                    _ => None,
                }
                .and_then(|(lo, hi)| (lo..=hi).take(named.len() + 1).find(|v| !named.contains(v)));
                let each = arms
                    .iter()
                    .filter_map(|arm| match &arm.tags {
                        Some(tags) => tags.first().copied(),
                        None => rest,
                    })
                    .collect();
                out.push((path.iter().copied().chain([0]).collect(), each));
                if let Some((arm, rest)) = chosen(arms, items) {
                    let mut env = env.clone();
                    env.insert(name.clone(), items[0].number().unwrap_or(0));
                    for (index, (shape, item)) in arm.parts.iter().zip(rest).enumerate() {
                        path.push(1 + index);
                        walk(shape, item, &env, path, out);
                        path.pop();
                        if let Some(names) = arm.binds.get(index).filter(|names| !names.is_empty())
                        {
                            bind(names, item, &mut env);
                        }
                    }
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
            (Shape::Tuple { parts, .. }, names) if names.len() == parts.len() => {
                for (part, name) in names.iter().enumerate() {
                    if is_size(name) && matches!(parts[part], Shape::Int { .. }) {
                        out.push(vec![index, part]);
                    }
                }
            }
            _ => {}
        }
        inner_sizes(
            spec,
            &field.shape,
            &case.values[index],
            &mut vec![index],
            &mut out,
        );
    }
    out.into_iter()
        .filter_map(|path| all.iter().find(|(p, _)| *p == path).cloned())
        .collect()
}

/// The sizes read inside a value: the `k` of each pass of a loop that
/// reads `k`, then `k` values.
fn inner_sizes(
    spec: &InputSpec,
    shape: &Shape,
    value: &Value,
    path: &mut Path,
    out: &mut Vec<Path>,
) {
    match (shape, value) {
        (Shape::Array { elem, .. }, Value::Seq(items)) => {
            for (index, item) in items.iter().enumerate() {
                path.push(index);
                inner_sizes(spec, elem, item, path, out);
                path.pop();
            }
        }
        (Shape::Tuple { parts, binds }, Value::Tuple(items)) => {
            for (index, (part, item)) in parts.iter().zip(items).enumerate() {
                path.push(index);
                match binds.get(index).map(Vec::as_slice) {
                    Some([Some(name)])
                        if spec.sizes.contains(name) && matches!(part, Shape::Int { .. }) =>
                    {
                        out.push(path.clone())
                    }
                    _ => inner_sizes(spec, part, item, path, out),
                }
                path.pop();
            }
        }
        (Shape::Choice { arms, .. }, Value::Tuple(items)) => {
            if let Some((arm, rest)) = chosen(arms, items) {
                let shape = Shape::Tuple {
                    parts: arm.parts.clone(),
                    binds: arm.binds.clone(),
                };
                let mut sub = Vec::new();
                inner_sizes(
                    spec,
                    &shape,
                    &Value::Tuple(rest.to_vec()),
                    &mut Vec::new(),
                    &mut sub,
                );
                for mut inner in sub {
                    inner[0] += 1;
                    out.push(path.iter().copied().chain(inner).collect());
                }
            }
        }
        _ => {}
    }
}

/// Where the size `name` is read: its field, and its part of a tuple.
fn size_path(spec: &InputSpec, name: &str) -> Option<Path> {
    spec.fields.iter().enumerate().find_map(|(index, field)| {
        let part = field
            .names
            .iter()
            .position(|n| n.as_deref() == Some(name))?;
        match &field.shape {
            Shape::Int { .. } => Some(vec![index]),
            Shape::Tuple { parts, .. } if parts.len() == field.names.len() => {
                Some(vec![index, part])
            }
            _ => None,
        }
    })
}

/// The one size an expression such as `n` or `m - 1` is made from.
fn sole_size(spec: &InputSpec, expr: &cpl_syntax::ast::Expr) -> Option<Path> {
    let mut names = std::collections::HashSet::new();
    names_in(expr, &mut names);
    let mut sizes = names.iter().filter(|name| spec.sizes.contains(*name));
    let name = sizes.next()?;
    sizes.next().is_none().then(|| size_path(spec, name))?
}

/// `value` with each number of `shape` that is a vertex of `lo..=hi`
/// renamed by `rename`; `None` when `rename` refuses one.
fn rename_vertices(
    shape: &Shape,
    value: &Value,
    ends: (&cpl_syntax::ast::Expr, &cpl_syntax::ast::Expr),
    rename: &dyn Fn(i128) -> Option<i128>,
) -> Option<Value> {
    match (shape, value) {
        (Shape::Int { lo, hi }, Value::Int(v)) if (lo, hi) == ends => Some(Value::Int(rename(*v)?)),
        (Shape::Array { elem, .. }, Value::Seq(items)) => items
            .iter()
            .map(|item| rename_vertices(elem, item, ends, rename))
            .collect::<Option<_>>()
            .map(Value::Seq),
        (Shape::Tuple { parts: shapes, .. }, Value::Tuple(items)) => shapes
            .iter()
            .zip(items)
            .map(|(shape, item)| rename_vertices(shape, item, ends, rename))
            .collect::<Option<_>>()
            .map(Value::Tuple),
        (Shape::Choice { arms, .. }, Value::Tuple(items)) => match chosen(arms, items) {
            Some((arm, rest)) => std::iter::once(Some(items[0].clone()))
                .chain(
                    arm.parts
                        .iter()
                        .zip(rest)
                        .map(|(shape, item)| rename_vertices(shape, item, ends, rename)),
                )
                .collect::<Option<_>>()
                .map(Value::Tuple),
            None => Some(value.clone()),
        },
        _ => Some(value.clone()),
    }
}

/// `case` with the edges of field `field` replaced by `edges`, renamed
/// by `rename`, and every other value that is one of its vertices too.
fn with_vertices_renamed(
    spec: &InputSpec,
    case: &Case,
    field: usize,
    edges: &[Value],
    rename: &dyn Fn(i128) -> Option<i128>,
) -> Option<Case> {
    let Shape::Graph { lo, hi, .. } = &spec.fields[field].shape else {
        return None;
    };
    let mut renamed = case.clone();
    for (index, (other, value)) in spec.fields.iter().zip(&case.values).enumerate() {
        renamed.values[index] = if index == field {
            let edges = edges.iter().map(|edge| {
                let Value::Tuple(ends) = edge else {
                    return None;
                };
                let mut ends = ends.clone();
                for end in &mut ends[..2] {
                    *end = end.with(rename(end.number()?)?);
                }
                Some(Value::Tuple(ends))
            });
            Value::Seq(edges.collect::<Option<_>>()?)
        } else {
            rename_vertices(&other.shape, value, (lo, hi), rename)?
        };
    }
    Some(renamed)
}

/// `case` without vertex `v` of the graph read by field `field`: its
/// edges go, the vertices above it move down one, and the sizes that
/// count the vertices and the edges drop to match.
fn without_vertex(spec: &InputSpec, case: &Case, field: usize, v: i128) -> Option<Case> {
    let Shape::Graph { hi, len, .. } = &spec.fields[field].shape else {
        return None;
    };
    let (vertices, edges) = (sole_size(spec, hi)?, sole_size(spec, len)?);
    (vertices != edges).then_some(())?;
    let Value::Seq(items) = &case.values[field] else {
        return None;
    };
    let touches = |item: &Value| matches!(item, Value::Tuple(ends) if ends[..2].iter().any(|end| end.number() == Some(v)));
    let kept: Vec<Value> = items
        .iter()
        .filter(|item| !touches(item))
        .cloned()
        .collect();
    let gone = (items.len() - kept.len()) as i128;
    let down = |w: i128| match w.cmp(&v) {
        std::cmp::Ordering::Less => Some(w),
        std::cmp::Ordering::Equal => None,
        std::cmp::Ordering::Greater => Some(w - 1),
    };
    let mut smaller = with_vertices_renamed(spec, case, field, &kept, &down)?;
    for (path, by) in [(vertices, 1), (edges, gone)] {
        let slot = at(&mut smaller, &path);
        *slot = slot.with(slot.number()? - by);
    }
    Some(smaller)
}

/// `case` with the vertices of the graph read by field `field` renamed
/// `lo, lo + 1, ..` in the order the edges first name them, the rest
/// after them in order: the same graph, written plainer.
fn renumbered(spec: &InputSpec, case: &Case, field: usize, lo: i128, hi: i128) -> Option<Case> {
    let Value::Seq(items) = &case.values[field] else {
        return None;
    };
    let mut order: Vec<i128> = Vec::new();
    for item in items {
        let Value::Tuple(ends) = item else {
            return None;
        };
        for end in &ends[..2] {
            let end = end.number()?;
            if !order.contains(&end) {
                order.push(end);
            }
        }
    }
    let unseen: Vec<i128> = (lo..=hi).filter(|v| !order.contains(v)).collect();
    order.extend(unseen);
    let name = |v: i128| Some(lo + order.iter().position(|w| *w == v)? as i128);
    with_vertices_renamed(spec, case, field, items, &name)
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
/// Swapping vertex names is tried among the first this many vertices.
const SWAP_LIMIT: i128 = 16;

/// The seeds tried for each `read(gen: f)` value while shrinking.
const GEN_SEEDS: u64 = 16;

/// `case`'s values with every drawn one to be drawn again from `seed`.
fn drawn_with(spec: &InputSpec, case: &Case, seed: u64) -> Vec<Value> {
    let fresh = || Value::Drawn {
        seed,
        text: String::new(),
    };
    spec.fields
        .iter()
        .zip(&case.values)
        .map(|(field, value)| match field.shape {
            Shape::Gen { .. } => fresh(),
            _ => value.clone(),
        })
        .collect()
}

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
            return true;
        }
        // A drawn value made from other values is drawn anew with them:
        // a smaller `n` may fail only with another word of that length.
        for seed in 0..GEN_SEEDS {
            let reseeded = Case {
                values: drawn_with(spec, &candidate, seed),
                ..candidate.clone()
            };
            let Some(reseeded) = fit(spec, &reseeded).filter(|c| *c != candidate) else {
                continue;
            };
            if measure(&reseeded) >= measure(best) || left == 0 {
                continue;
            }
            left -= 1;
            if still_fails(&reseeded) {
                *best = reseeded;
                return true;
            }
        }
        false
    };
    // Lowers the number at `path` as far as the failure allows.
    let lower = |best: &mut Case,
                 path: &Path,
                 lo: i128,
                 attempt: &mut dyn FnMut(&mut Case, Case) -> bool|
     -> bool {
        let Some(start) = number_at(best, path) else {
            return false;
        };
        // The low end first: it may be the only value below `start`
        // that fits, as `none` is below a gap in `?(1..=n)`.
        if lo < start {
            let mut candidate = best.clone();
            let slot = at(&mut candidate, path);
            *slot = slot.with(lo);
            if attempt(best, candidate) {
                return true;
            }
        }
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
            while let Some(current) = number_at(&best, &path) {
                if current <= lo {
                    break;
                }
                let Some(smaller) = without_element(spec, &best, &path, position) else {
                    break;
                };
                if attempt(&mut best, smaller) {
                    improved = true;
                } else {
                    position += 1;
                }
            }
        }
        // 1a. Drawn values: another seed may draw a smaller one.
        if spec
            .fields
            .iter()
            .any(|f| matches!(f.shape, Shape::Gen { .. }))
        {
            for seed in 0..GEN_SEEDS {
                let candidate = Case {
                    values: drawn_with(spec, &best, seed),
                    ..best.clone()
                };
                improved |= attempt(&mut best, candidate);
            }
        }
        // 2a. Graphs: delete one vertex, with its edges.
        for field in 0..best.values.len() {
            let Shape::Graph { lo, hi, .. } = &spec.fields[field].shape else {
                continue;
            };
            let Some(lo) = eval(lo, &spec.consts) else {
                continue;
            };
            let mut v = lo;
            while let Value::Seq(items) = &best.values[field] {
                let top = items.iter().filter_map(|item| match item {
                    Value::Tuple(ends) => ends[..2].iter().filter_map(Value::number).max(),
                    _ => None,
                });
                if top.max().is_none_or(|top| v > top) {
                    break;
                }
                let smaller = without_vertex(spec, &best, field, v);
                if smaller.is_some_and(|smaller| attempt(&mut best, smaller)) {
                    improved = true;
                } else {
                    v += 1;
                }
            }
            let mut env = spec.consts.clone();
            for (field, value) in spec.fields.iter().zip(&best.values) {
                bind(&field.names, value, &mut env);
            }
            let Some(hi) = eval(hi, &env) else {
                continue;
            };
            if let Some(plainer) = renumbered(spec, &best, field, lo, hi) {
                improved |= attempt(&mut best, plainer);
            }
            // Two names swapped: a star whose centre is not vertex `lo`
            // is written plainer with the centre renamed `lo`.
            for a in lo..=hi.min(lo + SWAP_LIMIT) {
                for b in a + 1..=hi.min(lo + SWAP_LIMIT) {
                    let Value::Seq(items) = best.values[field].clone() else {
                        break;
                    };
                    let swap = |v: i128| {
                        Some(if v == a {
                            b
                        } else if v == b {
                            a
                        } else {
                            v
                        })
                    };
                    let swapped = with_vertices_renamed(spec, &best, field, &items, &swap);
                    if let Some(swapped) = swapped {
                        improved |= attempt(&mut best, swapped);
                    }
                }
            }
        }
        // 2b. Undirected edges, low end first.
        for field in 0..best.values.len() {
            let undirected = match &spec.fields[field].shape {
                Shape::Tree { .. } => true,
                Shape::Graph { kind, .. } => !kind.directed(),
                _ => false,
            };
            let Value::Seq(items) = &best.values[field] else {
                continue;
            };
            if !undirected {
                continue;
            }
            let flipped = items
                .iter()
                .map(|item| match item {
                    Value::Tuple(ends) if ends[0].number() > ends[1].number() => {
                        let mut ends = ends.clone();
                        ends.swap(0, 1);
                        Value::Tuple(ends)
                    }
                    other => other.clone(),
                })
                .collect();
            let mut plainer = best.clone();
            plainer.values[field] = Value::Seq(flipped);
            improved |= attempt(&mut best, plainer);
        }
        // 2c. Words: drop one letter at a time.
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
        // 3. Values, toward their lower bound; a tag first to the arm
        //    that reads least.
        for (path, each) in tags(spec, &best) {
            for tag in each {
                // An earlier switch may have removed this tag.
                if !tags(spec, &best).iter().any(|(p, _)| *p == path) {
                    break;
                }
                let mut candidate = best.clone();
                let slot = at(&mut candidate, &path);
                *slot = slot.with(tag);
                improved |= attempt(&mut best, candidate);
            }
        }
        for (path, lo) in numbers(spec, &best) {
            // An earlier step may have removed this number.
            let exists = numbers(spec, &best).iter().any(|(p, _)| *p == path);
            if exists {
                improved |= lower(&mut best, &path, lo, &mut attempt);
            }
        }
        // 3b. Equal values, together.
        for value in distinct_numbers(spec, std::slice::from_ref(&best)) {
            let mut cases = vec![best.clone()];
            improved |= lower_equal(spec, &mut cases, value, &mut |candidate| {
                let [candidate] = <[Case; 1]>::try_from(candidate).ok()?;
                attempt(&mut best, candidate).then(|| vec![best.clone()])
            });
        }
        // 3c. A value lowered by one as an element goes: a failure that
        //    needs `m` values past a bound of `m` stops both steps alone.
        if !improved {
            'pairs: for (size, lo) in sizes(spec, &best) {
                for position in 0.. {
                    if number_at(&best, &size).is_none_or(|current| current <= lo) {
                        break;
                    }
                    let Some(smaller) = without_element(spec, &best, &size, position) else {
                        break;
                    };
                    for (path, lo) in numbers(spec, &smaller) {
                        let Some(value) = number_at(&smaller, &path).filter(|&v| v > lo) else {
                            continue;
                        };
                        let mut candidate = smaller.clone();
                        let slot = at(&mut candidate, &path);
                        *slot = slot.with(value - 1);
                        if attempt(&mut best, candidate) {
                            improved = true;
                            break 'pairs;
                        }
                    }
                }
            }
        }
        if !improved {
            break;
        }
    }
    best
}

/// `case` with the size at `path` one smaller and element `position`
/// dropped from each array that gets shorter, at any depth: a smaller
/// `k` shortens every permutation in a list of them. `None` when no array
/// is shorter or `position` is past the end of them.
fn without_element(spec: &InputSpec, case: &Case, path: &Path, position: usize) -> Option<Case> {
    let current = number_at(case, path)?;
    let mut smaller = case.clone();
    *at(&mut smaller, path) = Value::Int(current - 1);
    let fitted = fit(spec, &smaller)?;
    let mut any = false;
    for (field, new) in fitted.values.iter().enumerate() {
        any |= drop_where_shorter(
            &case.values[field],
            new,
            &mut smaller.values[field],
            position,
        );
    }
    any.then_some(smaller)
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
        // Equal values in different test cases, together.
        for value in distinct_numbers(&scoped, &best.cases) {
            let mut cases = best.cases.clone();
            let lowered = lower_equal(&scoped, &mut cases, value, &mut |candidate| {
                if left == 0 {
                    return None;
                }
                left -= 1;
                let fitted: Vec<Case> = candidate
                    .iter()
                    .map(|case| fit(&scoped, case))
                    .collect::<Option<_>>()?;
                still_fails(&with(fitted.clone())).then_some(fitted)
            });
            if lowered {
                best.cases = cases;
            }
        }
        if best.weight() >= before || left == 0 {
            return best;
        }
    }
}

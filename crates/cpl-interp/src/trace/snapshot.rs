//! How a variable is drawn and the snapshot of its value at a step.

use super::*;

pub(super) fn kind_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Scalar => "scalar",
        Kind::Array | Kind::Parents => "array",
        Kind::Grid => "grid",
        Kind::Dp => "dp",
        Kind::Adjacency | Kind::Csr => "graph",
        Kind::Rooted => "tree",
        Kind::Json => "json",
    }
}

/// One value the viewer shows in a cell.
pub(super) fn is_cell(ty: &Ty, program: &Program) -> bool {
    match ty {
        Ty::Int | Ty::Float | Ty::Bool | Ty::Char | Ty::Str | Ty::Pos(_) | Ty::Bound(_) => true,
        Ty::Opt(inner) => is_cell(inner, program),
        Ty::Enum(id) => program.enums[*id]
            .variants
            .iter()
            .all(|(_, fields)| fields.is_empty()),
        _ => false,
    }
}

/// The domain a neighbour list leads into: `V` in `List<V>` and in
/// `List<(V, Int)>`.
pub(super) fn target(elem: &Ty) -> Option<usize> {
    let Ty::List(item) = elem else {
        return None;
    };
    match &**item {
        Ty::Pos(d) => Some(*d),
        Ty::Tuple(fields) => match fields.first() {
            Some(Ty::Pos(d)) => Some(*d),
            _ => None,
        },
        Ty::Record(fields) => match fields.first() {
            Some((_, Ty::Pos(d))) => Some(*d),
            _ => None,
        },
        _ => None,
    }
}

pub(super) fn labels(dim: Dim, len: i64, view: &View) -> String {
    let lo = match dim {
        Dim::Pos(d) | Dim::Bound(d) => view.domains[d].0,
        Dim::Sym(_) => 0,
        Dim::Enum(_) => 0,
        Dim::Int => 0,
        Dim::IntFrom(first) => first,
    };
    let mut out = String::from("[");
    for i in 0..len.clamp(0, 1000) {
        if i > 0 {
            out.push(',');
        }
        out.push_str(&(lo + i).to_string());
    }
    out.push(']');
    out
}

pub(super) fn dim_len(dim: Dim, view: &View) -> Option<i64> {
    match dim {
        Dim::Pos(d) => Some(view.domains[d].1),
        Dim::Bound(d) => Some(view.domains[d].1 + 1),
        Dim::Sym(d) => {
            let n = view.domains[d].1;
            Some(n * (n + 1) / 2)
        }
        Dim::Enum(e) => Some(view.program.enums[e].variants.len() as i64),
        Dim::Int | Dim::IntFrom(_) => None,
    }
}

pub(super) fn dim_name(dim: Dim, view: &View<'_>) -> Option<String> {
    match dim {
        Dim::Pos(d) => Some(view.program.domains[d].clone()),
        Dim::Bound(d) => Some(format!("{}.cuts", view.program.domains[d])),
        Dim::Sym(d) => Some(format!("Sym<{}>", view.program.domains[d])),
        Dim::Enum(e) => Some(view.program.enums[e].name.clone()),
        Dim::Int | Dim::IntFrom(_) => None,
    }
}

/// How a variable of type `ty` is drawn, and the rest of its descriptor.
pub(super) fn classify(ty: &Ty, view: &View) -> (Kind, String) {
    let program = view.program;
    match ty {
        _ if is_cell(ty, program) => (Kind::Scalar, String::new()),
        Ty::Array { dims, elem } if dims.len() == 1 => {
            let dim = dims[0];
            if let (Dim::Pos(d), Some(to)) = (dim, target(elem)) {
                if d == to {
                    return (Kind::Adjacency, ",\"directed\":true".to_string());
                }
            }
            let mut extra = String::new();
            if let Some(len) = dim_len(dim, view) {
                extra = format!(",\"labels\":{}", labels(dim, len, view));
            }
            match &**elem {
                Ty::Opt(inner) if matches!(dim, Dim::Pos(d) if **inner == Ty::Pos(d)) => {
                    (Kind::Parents, extra)
                }
                elem if is_cell(elem, program) => (Kind::Array, extra),
                // A row of named cells per index: the viewer draws a table
                // with the index's labels down the side.
                elem if field_names(elem, program).is_some()
                    && field_tys(elem, program)
                        .iter()
                        .all(|ty| is_cell(ty, program)) =>
                {
                    (Kind::Json, extra)
                }
                _ => (Kind::Json, String::new()),
            }
        }
        Ty::Array { dims, elem } if dims.len() == 2 && is_cell(elem, program) => {
            let mut extra = String::new();
            for (dim, labels_key, domain_key) in [
                (dims[0], "rowLabels", "rowDomain"),
                (dims[1], "colLabels", "colDomain"),
            ] {
                if let Some(len) = dim_len(dim, view) {
                    extra += &format!(",\"{labels_key}\":{}", labels(dim, len, view));
                }
                if let Some(name) = dim_name(dim, view) {
                    extra += &format!(",\"{domain_key}\":");
                    string(&mut extra, &name);
                }
            }
            let kind = match &**elem {
                Ty::Char | Ty::Bool | Ty::Enum(_) => Kind::Grid,
                Ty::Opt(inner) if matches!(**inner, Ty::Char | Ty::Bool | Ty::Enum(_)) => {
                    Kind::Grid
                }
                _ => Kind::Dp,
            };
            (kind, extra)
        }
        Ty::List(elem) if is_cell(elem, program) => (Kind::Array, String::new()),
        Ty::Struct(id) => {
            let def = &program.structs[*id];
            let field = |name: &str| {
                def.fields
                    .iter()
                    .find(|(field, _)| field == name)
                    .map(|(_, ty)| ty)
            };
            // Drawn by shape, not by name, so a program's own struct laid
            // out the same way is drawn the same way: offsets into a flat
            // list of arcs that each start with a vertex is a graph, and
            // a root beside a parent per vertex is a tree.
            let arcs_from = |ty: &Ty| match ty {
                Ty::List(arc) => matches!(field_tys(arc, program).first(), Some(Ty::Pos(_))),
                _ => false,
            };
            let parent_per = |ty: &Ty| match ty {
                Ty::Array { elem, .. } => {
                    matches!(&**elem, Ty::Opt(of) if matches!(**of, Ty::Pos(_)))
                }
                _ => false,
            };
            if field("start") == Some(&Ty::List(Box::new(Ty::Int)))
                && field("arcs").is_some_and(arcs_from)
            {
                (Kind::Csr, ",\"directed\":true".to_string())
            } else if matches!(field("root"), Some(Ty::Pos(_)))
                && field("par").is_some_and(parent_per)
            {
                (Kind::Rooted, String::new())
            } else {
                (Kind::Json, String::new())
            }
        }
        _ => (Kind::Json, String::new()),
    }
}

/// How many values `value` holds, to keep huge ones out of a trace.
pub(super) fn cells(value: &Value) -> usize {
    match value {
        Value::Array(array) => match &array.data {
            crate::value::ArrayData::Values(v) => all_cells(v.iter()),
            _ => array.len(),
        },
        Value::List(items) => 1 + all_cells(items.iter()),
        Value::Enum(_, items) => 1 + items.iter().map(cells).sum::<usize>(),
        Value::Tuple(TupleVal::OneInt(_)) => 2,
        Value::Tuple(TupleVal::Boxed(items)) => 1 + items.iter().map(cells).sum::<usize>(),
        Value::Map(entries) => entries.iter().map(|(k, v)| cells(k) + cells(v)).sum(),
        Value::Set(items) => all_cells(items.iter()),
        Value::Opt(OptVal::Int(_)) => 1,
        Value::Opt(OptVal::Boxed(inner)) => cells(inner),
        Value::Str(bytes) => 1 + bytes.len() / 64,
        _ => 1,
    }
}

/// The cells of a collection's items, which share a type: one each when
/// they are numbers or the like, so a long list costs nothing to measure.
pub(super) fn all_cells<'v>(mut items: impl ExactSizeIterator<Item = &'v Value>) -> usize {
    let n = items.len();
    match items.next() {
        None => 0,
        Some(
            Value::Int(_)
            | Value::I128(_)
            | Value::U64(_)
            | Value::Float(_)
            | Value::Bool(_)
            | Value::Char(_)
            | Value::Range(..)
            | Value::Range128(..),
        ) => n,
        Some(first) => cells(first) + items.map(cells).sum::<usize>(),
    }
}

/// `text` as a JSON string safe for embedding in HTML `<script>` tags.
pub fn string(out: &mut String, text: &str) {
    let _ = cpl_json::write_script_safe_string(out, text);
}

pub(super) fn bytes_text(bytes: &[u8]) -> String {
    bytes.iter().map(|b| *b as char).collect()
}

/// The named `params` among the locals, as a JSON object.
pub(crate) fn args_json(view: &View, params: &[&str]) -> String {
    let mut out = String::from("{");
    for (name, ty, value) in view
        .locals
        .iter()
        .filter(|(name, ..)| params.contains(name))
    {
        if out.len() > 1 {
            out.push(',');
        }
        string(&mut out, name);
        out.push(':');
        out.push_str(&json_of(value, ty, view));
    }
    out.push('}');
    out
}

/// A graph value of the viewer from its vertices' labels and its edges.
pub(super) fn graph(
    nodes: impl Iterator<Item = i64>,
    edges: &[(i64, i64, Option<String>)],
) -> String {
    let mut out = String::from("{\"nodes\":[");
    for (i, node) in nodes.enumerate() {
        if i > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{node},\"label\":\"{node}\"}}");
    }
    out.push_str("],\"edges\":[");
    for (i, (from, to, weight)) in edges.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{i},\"from\":{from},\"to\":{to}");
        if let Some(weight) = weight {
            out += &format!(",\"weight\":{weight}");
        }
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// The edges a parent array holds, parent to child, when it is one
/// rooted tree: a single root and no cycle.
pub(super) fn tree_edges(parents: &[Option<i64>]) -> Option<Vec<(i64, i64, Option<String>)>> {
    if parents.iter().filter(|p| p.is_none()).count() != 1 {
        return None;
    }
    let n = parents.len();
    // Every vertex must reach the root.
    let mut state = vec![0u8; n];
    for start in 0..n {
        let mut path = Vec::new();
        let mut at = start;
        while state[at] == 0 {
            state[at] = 1;
            path.push(at);
            match parents[at] {
                Some(parent) if (0..n as i64).contains(&parent) => at = parent as usize,
                Some(_) => return None,
                None => break,
            }
        }
        if state[at] == 1 && parents[at].is_some() {
            return None;
        }
        for v in path {
            state[v] = 2;
        }
    }
    Some(
        parents
            .iter()
            .enumerate()
            .filter_map(|(child, parent)| parent.map(|parent| (parent, child as i64, None)))
            .collect(),
    )
}

/// A parent array as ordinals: `None` at a root.
pub(super) fn parents_of(value: &Value) -> Option<Vec<Option<i64>>> {
    let Value::Array(array) = value else {
        return None;
    };
    let vals = array.to_values();
    Some(
        vals.iter()
            .map(|cell| match cell {
                Value::Opt(OptVal::Int(parent)) => Some(*parent),
                Value::Opt(OptVal::Boxed(parent)) => Some(parent.int()),
                _ => None,
            })
            .collect(),
    )
}

/// Past this a JSON number is no longer exact.
pub(crate) const MAX_EXACT: u64 = 9_007_199_254_740_991;

/// Item `index` of a one-dimensional array of `Int`s, when its JSON is
/// the number itself.
pub(super) fn int_item(value: &Value, ty: &Ty, index: usize) -> Option<i64> {
    let (Value::Array(array), Ty::Array { dims, elem }) = (value, ty) else {
        return None;
    };
    if dims.len() != 1 || !matches!(**elem, Ty::Int) || array.len() > MAX_CELLS {
        return None;
    }
    array
        .get_int(index)
        .filter(|v| v.unsigned_abs() <= MAX_EXACT)
}

/// Item `index` of a one-dimensional array or list, as JSON, as it is in
/// the JSON of the whole.
pub(super) fn item_json(value: &Value, ty: &Ty, index: usize, view: &View) -> Option<String> {
    let (item, elem) = match (value, ty) {
        (Value::Array(array), Ty::Array { dims, elem }) if dims.len() == 1 => {
            (array.get(index)?, &**elem)
        }
        (Value::List(items), Ty::List(elem)) => (items.get(index)?.clone(), &**elem),
        _ => return None,
    };
    let scalar = matches!(
        elem,
        Ty::Int | Ty::I128 | Ty::U64 | Ty::Float | Ty::Bool | Ty::Char | Ty::Pos(_) | Ty::Bound(_)
    );
    let len = match value {
        Value::Array(array) => array.len(),
        Value::List(items) => items.len() + 1,
        _ => unreachable!("matched above"),
    };
    if (if scalar { len } else { cells(value) }) > MAX_CELLS {
        return None;
    }
    let mut json = String::new();
    render(&mut json, &item, elem, view, Mode::Json);
    Some(json)
}

/// The most items of a list or array a window draws: what the viewer shows.
const WINDOW: usize = 500;
/// The most rows, and columns, of a grid a window draws.
const WINDOW_SIDE: usize = 100;

/// Whether `value` is too large to keep whole and is drawn by [`window`].
pub(super) fn drawn_by_window(value: &Value, ty: &Ty, kind: Kind) -> bool {
    let shaped = match (kind, value, ty) {
        (Kind::Array, Value::Array(array), Ty::Array { .. }) => array.dims.len() == 1,
        (Kind::Array, Value::List(_), Ty::List(_)) => true,
        (Kind::Grid | Kind::Dp, Value::Array(array), Ty::Array { .. }) => array.dims.len() == 2,
        _ => false,
    };
    shaped && cells(value) > MAX_CELLS
}

/// The first cells of a [`drawn_by_window`] value: its first [`WINDOW`] items, or
/// the top-left [`WINDOW_SIDE`] square of a grid.
fn window(value: &Value, ty: &Ty, view: &View) -> Option<String> {
    let mut out = String::from("[");
    let item = |out: &mut String, at: usize, item: &Value, elem: &Ty| {
        if at > 0 {
            out.push(',');
        }
        render(out, item, elem, view, Mode::Json);
    };
    match (value, ty) {
        (Value::List(items), Ty::List(elem)) => {
            for (at, x) in items.iter().take(WINDOW).enumerate() {
                item(&mut out, at, x, elem);
            }
        }
        (Value::Array(array), Ty::Array { elem, .. }) if array.dims.len() == 1 => {
            for at in 0..array.len().min(WINDOW) {
                item(&mut out, at, &array.get(at).unwrap_or(Value::Unit), elem);
            }
        }
        (Value::Array(array), Ty::Array { elem, .. }) => {
            let (rows, cols) = (array.dims[0] as usize, array.dims[1] as usize);
            for r in 0..rows.min(WINDOW_SIDE) {
                out.push_str(if r > 0 { ",[" } else { "[" });
                for c in 0..cols.min(WINDOW_SIDE) {
                    item(
                        &mut out,
                        c,
                        &array.get(r * cols + c).unwrap_or(Value::Unit),
                        elem,
                    );
                }
                out.push(']');
            }
        }
        _ => return None,
    }
    out.push(']');
    Some(out)
}

/// The snapshot of a variable, and for a `[V]?V` its tree form when it
/// is a tree.
pub(super) fn snapshot(
    value: &Value,
    ty: &Ty,
    kind: Kind,
    view: &View,
) -> (String, Option<String>) {
    if let Some(window) = drawn_by_window(value, ty, kind)
        .then(|| window(value, ty, view))
        .flatten()
    {
        return (window, None);
    }
    if cells(value) > MAX_CELLS {
        let mut out = String::new();
        // Too large to draw: an empty value of the right shape, so the
        // viewer still takes the trace.
        match kind {
            Kind::Array | Kind::Parents | Kind::Grid | Kind::Dp => out.push_str("[]"),
            Kind::Adjacency | Kind::Csr | Kind::Rooted => {
                out.push_str("{\"nodes\":[],\"edges\":[]}")
            }
            Kind::Scalar | Kind::Json => string(&mut out, &format!("{} values", cells(value))),
        }
        return (out, None);
    }
    let plain = || {
        let mut out = String::new();
        render(&mut out, value, ty, view, Mode::Json);
        out
    };
    let lo_of = |dim: Option<&Dim>| match dim {
        Some(Dim::Pos(d)) => view.domains[*d].0,
        _ => 0,
    };
    match (kind, value, ty) {
        (Kind::Adjacency, Value::Array(array), Ty::Array { dims, elem }) => {
            let lo = lo_of(dims.first());
            let weighted = match &**elem {
                Ty::List(item) => field_tys(item, view.program).get(1).cloned(),
                _ => None,
            };
            let mut edges = Vec::new();
            let vals = array.to_values();
            for (from, list) in vals.iter().enumerate() {
                let Value::List(items) = list else { continue };
                for item in items.iter() {
                    let (to, weight) = match item {
                        Value::Tuple(fields) => {
                            let mut scratch = [Value::Unit];
                            let fields = fields.as_slice(&mut scratch);
                            (
                                fields.first().map_or(0, Value::int),
                                match (&weighted, fields.get(1)) {
                                    (Some(ty @ (Ty::Int | Ty::Float)), Some(w)) => {
                                        Some(json_of(w, ty, view))
                                    }
                                    _ => None,
                                },
                            )
                        }
                        other => (other.int(), None),
                    };
                    edges.push((from as i64 + lo, to + lo, weight));
                }
            }
            (graph((0..array.len() as i64).map(|i| i + lo), &edges), None)
        }
        (Kind::Parents, _, Ty::Array { dims, .. }) => {
            let lo = lo_of(dims.first());
            let tree = parents_of(value).and_then(|parents| {
                let edges: Vec<_> = tree_edges(&parents)?
                    .into_iter()
                    .map(|(from, to, weight)| (from + lo, to + lo, weight))
                    .collect();
                // The root first: the viewer roots a tree at its first node.
                let root = parents.iter().position(Option::is_none)? as i64;
                let nodes = std::iter::once(root)
                    .chain((0..parents.len() as i64).filter(move |v| *v != root));
                Some(graph(nodes.map(|v| v + lo), &edges))
            });
            (plain(), tree)
        }
        (Kind::Csr, Value::Tuple(fields), Ty::Struct(id)) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let def = &view.program.structs[*id];
            let at = |name: &str| def.fields.iter().position(|(field, _)| field == name);
            let (Some(start), Some(arcs)) = (at("start"), at("arcs")) else {
                return (graph(std::iter::empty(), &[]), None);
            };
            let lo = match &def.fields[arcs].1 {
                Ty::List(item) => match field_tys(item, view.program).first() {
                    Some(Ty::Pos(d)) => view.domains[*d].0,
                    _ => 0,
                },
                _ => 0,
            };
            let (Value::List(start), Value::List(arcs)) = (&fields[start], &fields[arcs]) else {
                return (graph(std::iter::empty(), &[]), None);
            };
            let n = start.len().saturating_sub(1);
            let mut edges = Vec::new();
            for from in 0..n {
                for arc in start[from].int()..start[from + 1].int() {
                    if let Some(Value::Tuple(arc)) = arcs.get(arc as usize) {
                        edges.push((
                            from as i64 + lo,
                            arc.get(0).map_or(0, |v| v.int()) + lo,
                            None,
                        ));
                    }
                }
            }
            (graph((0..n as i64).map(|i| i + lo), &edges), None)
        }
        (Kind::Rooted, Value::Tuple(fields), Ty::Struct(id)) => {
            let mut scratch = [Value::Unit];
            let fields = fields.as_slice(&mut scratch);
            let def = &view.program.structs[*id];
            let at = |name: &str| def.fields.iter().position(|(field, _)| field == name);
            let empty = (graph(std::iter::empty(), &[]), None);
            let (Some(root), Some(par)) = (at("root"), at("par")) else {
                return empty;
            };
            let lo = match &def.fields[root].1 {
                Ty::Pos(d) => view.domains[*d].0,
                _ => 0,
            };
            let Some(parents) = parents_of(&fields[par]) else {
                return empty;
            };
            let root = fields[root].int();
            // Vertices the root does not reach are left out: a tree has
            // one root.
            let edges: Vec<_> = parents
                .iter()
                .enumerate()
                .filter_map(|(child, parent)| {
                    parent.map(|parent| (parent + lo, child as i64 + lo, None))
                })
                .collect();
            let nodes = std::iter::once(root).chain(
                (0..parents.len() as i64).filter(|v| *v != root && parents[*v as usize].is_some()),
            );
            (graph(nodes.map(|v| v + lo), &edges), None)
        }
        (Kind::Grid | Kind::Dp, Value::Array(array), _)
            if array.dims.len() != 2 || array.is_empty() =>
        {
            ("[]".to_string(), None)
        }
        _ => (plain(), None),
    }
}

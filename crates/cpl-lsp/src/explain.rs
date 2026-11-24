//! The storage section `cpl explain` opens with, shared by the CLI and
//! the playground: the checker's representation query, as hover reads it.

/// Each top-level value's representation, and the most bytes the fixed-size
/// ones take with the names of those that grow: the checker's query, the
/// data hover shows.
#[allow(clippy::type_complexity)]
pub fn storage(
    db: &mut cpl_check::Database,
    path: &str,
) -> (
    Vec<(String, cpl_check::Representation)>,
    Option<(u128, Vec<String>)>,
) {
    let rows = db
        .symbols(path)
        .into_iter()
        .filter(|s| matches!(s.kind, cpl_check::DeclKind::Value))
        .filter_map(|s| Some((s.name, db.representation(path, s.name_span.start)?)))
        .collect();
    let total = db
        .storage(path)
        .map(|(bytes, grows)| (bytes, grows.into_iter().map(|(name, _)| name).collect()));
    (rows, total)
}

/// `cpl explain`'s types and memory section (spec/tools/explain.md).
pub fn storage_section(db: &mut cpl_check::Database, path: &str) -> String {
    let (rows, total) = storage(db, path);
    if rows.is_empty() {
        return format!("Storage for {path}: no top-level values\n");
    }
    let mut out = format!("Storage for {path} (at the input bounds):\n");
    let name_width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0);
    let ty_width = rows.iter().map(|(_, rep)| rep.ty.len()).max().unwrap_or(0);
    for (name, rep) in &rows {
        let line = format!(
            "  {name:name_width$}  {:ty_width$}  {}",
            rep.ty,
            describe(rep)
        );
        out += line.trim_end();
        out += "\n";
    }
    if let Some((bytes, grows)) = total {
        out += &format!("  peak {}", cpl_check::human_bytes(bytes));
        out += &judge_limit(db, path, bytes);
        if !grows.is_empty() {
            out += &format!(", not counting {}", grows.join(", "));
        }
        out += "\n";
    }
    out
}

/// How a peak compares with the memory limit of the judge the build is
/// for, worded as the editor's memory lens words it (Rjudge.7).
fn judge_limit(db: &cpl_check::Database, path: &str, bytes: u128) -> String {
    let text = db
        .file_text(path)
        .map(|text| text.to_string())
        .or_else(|| std::fs::read_to_string(path).ok())
        .unwrap_or_default();
    let Ok(judge) = cpl_judge::resolve(std::path::Path::new(path), &text, None) else {
        return String::new();
    };
    let (mb, name) = (judge.memory_mb, judge.name);
    match bytes > u128::from(mb) << 20 {
        true => format!(", over the {mb} MB limit on {name}"),
        false => format!(" of {mb} MB on {name}"),
    }
}

/// One value's bytes: per element and at most for an array, then where an
/// optional keeps `None` and a record's field offsets and padding.
pub fn describe(rep: &cpl_check::Representation) -> String {
    let bytes = |n: u64| cpl_check::human_bytes(n.into());
    let mut out = match (rep.bytes, rep.max_cells, rep.max_bytes) {
        (Some(per), Some(cells), Some(most)) => format!(
            "{} per element x {cells} = {}",
            bytes(per),
            cpl_check::human_bytes(most)
        ),
        (Some(per), None, _) if rep.ty.starts_with('[') => {
            format!("{} per element, no bound on the count", bytes(per))
        }
        (Some(each), _, _) => bytes(each),
        (None, _, _) => "on the heap, as large as it grows".to_string(),
    };
    match rep.none_is_spare {
        Some(true) => out += ", None is a spare value (no flag)",
        Some(false) => out += ", None takes a flag beside the value",
        None => {}
    }
    if !rep.field_offsets.is_empty() {
        let at: Vec<String> = rep.field_offsets.iter().map(|o| format!("@{o}")).collect();
        out += &format!(", fields {}", at.join(" "));
        if rep.padding > 0 {
            out += &format!(", {} padding", bytes(rep.padding));
        }
    }
    out
}

//! The lenses above a file and its fns: each runs `cpl` (`cpl.exec`) or
//! replays a call, so a browser, with no `cpl` to run, has none.

use super::*;

/// `capabilities` with `codeLensProvider`.
pub(super) fn advertise(mut caps: Value) -> Value {
    if let Some(caps) = caps.as_object_mut().and_then(|c| c.get_mut("capabilities")) {
        let lens = obj([("resolveProvider", false.into())]);
        caps.as_object_mut()
            .map(|c| c.insert("codeLensProvider".into(), lens));
    }
    caps
}

impl Server {
    pub(super) fn code_lens(&mut self, params: &Value) -> Result<Reply, (i32, String)> {
        let uri = uri_of(params)?;
        let text = self.docs.get(uri).map(String::as_str).unwrap_or("");
        let lenses = match (self.lenses, path_of_uri(uri)) {
            (lenses, Some(path)) => {
                let path = crate::uri::native_path(&path);
                let mut all = if lenses {
                    code_lenses(&path, text)
                } else {
                    Vec::new()
                };
                all.extend(memory_lens(&mut self.db, uri, &path, text).map(|(t, a)| (0, t, a)));
                all
            }
            _ => Vec::new(),
        };
        let mut items: Vec<Value> = lenses
            .into_iter()
            .map(|(line, title, args)| {
                let args = args.into_iter().map(Value::from).collect();
                lens_json(line, title, "cpl.exec", args)
            })
            .collect();
        if self.replay {
            for (line, name) in replay_lenses(text) {
                let args = vec![uri.into(), obj([("fn", name.into())])];
                let title = "$(debug-restart) Replay this call".to_string();
                items.push(lens_json(line, title, "cp.replayCall", args));
            }
        }
        Ok(Reply::result(Value::Array(items)))
    }
}

fn lens_json(line: u32, title: String, command: &str, args: Vec<Value>) -> Value {
    let at = position_json(LineCol { line, col: 0 });
    obj([
        ("range", obj([("start", at.clone()), ("end", at)])),
        (
            "command",
            obj([
                ("title", title.into()),
                ("command", command.into()),
                ("arguments", Value::Array(args)),
            ]),
        ),
    ])
}

/// The 0-based line and name of each top-level fn of the solution: not
/// its `@alt`, `@check`, `@interactor` or `@grader` tools.
fn replay_lenses(text: &str) -> Vec<(u32, String)> {
    const TOOLS: [&str; 6] = ["alt", "brute", "check", "checker", "interactor", "grader"];
    let parsed = cpl_syntax::parse_file(text);
    let mut out = Vec::new();
    for (base, item) in &parsed.items {
        for stmt in &item.stmts {
            let cpl_syntax::ast::StmtKind::Fn(f) = &stmt.kind else {
                continue;
            };
            if stmt
                .decorators
                .iter()
                .any(|d| TOOLS.contains(&d.name.text.as_str()))
            {
                continue;
            }
            let start = *base as usize + f.name.span.start as usize;
            let line = text.get(..start).unwrap_or(text).matches('\n').count() as u32;
            out.push((line, f.name.text.clone()));
        }
    }
    out
}

/// The samples (name, input) `cpl run --samples` runs on `file` and whether
/// `cpl test` has cases for it (`cpl_contest::problem`).
fn file_cases(file: &std::path::Path) -> (Vec<(String, String)>, bool) {
    let samples = cpl_contest::problem::samples(file)
        .into_iter()
        .map(|case| (case.name, case.input.display().to_string()))
        .collect();
    (samples, !cpl_contest::problem::cases(file).is_empty())
}

/// Each lens's 0-based line, title and `cpl` arguments: Run, Run on all,
/// Trace, Test and Stress above the file, a Stress lens above each fn
/// `cpl stress` judges by, and a Trace lens above every other fn but
/// `main`. The samples and cases are the ones `cpl run --samples` and
/// `cpl test` find (`cpl_contest::problem`, commands/R15).
fn code_lenses(path: &str, text: &str) -> Vec<(u32, String, Vec<String>)> {
    let file = std::path::Path::new(path);
    if file.parent().is_none() || file.file_stem().is_none() {
        return Vec::new();
    }
    let (samples, has_tests) = file_cases(file);
    let mut lenses = Vec::new();
    let with_input = |title: &str, mut args: Vec<String>| match samples.first() {
        Some((label, input)) => {
            args.insert(2, input.clone());
            (0, format!("{title} on {label}"), args)
        }
        None => (0, title.to_string(), args),
    };
    lenses.push(with_input("Run", vec!["run".into(), path.into()]));
    let count = samples.len();
    if count > 1 {
        let args = vec!["run".into(), path.into(), "--samples".into()];
        lenses.push((0, format!("Run on all {count} samples"), args));
    }
    lenses.push(with_input(
        "Trace",
        vec!["run".into(), path.into(), "--trace".into()],
    ));
    if has_tests {
        lenses.push((0, "Test".into(), vec!["test".into(), path.into()]));
    }
    if cpl_ir::has_stress_target(text) {
        lenses.push((0, "Stress".into(), vec!["stress".into(), path.into()]));
    }
    let parsed = cpl_syntax::parse_file(text);
    for (base, stmt) in (parsed.items.iter())
        .flat_map(|(base, item)| item.stmts.iter().map(move |s| (*base as usize, s)))
    {
        let cpl_syntax::ast::StmtKind::Fn(f) = &stmt.kind else {
            continue;
        };
        let judge = stmt
            .decorators
            .iter()
            .find(|d| cpl_ir::STRESS_DECORATORS.contains(&d.name.text.as_str()));
        let name = &f.name.text;
        let start = base
            + (stmt.decorators.first())
                .map_or(stmt.span.start, |d| d.span.start.min(stmt.span.start))
                as usize;
        let line = text.get(..start).unwrap_or(text).matches('\n').count() as u32;
        let Some(judge) = judge else {
            if name != "main" {
                let trace = vec!["run".into(), path.into(), "--trace".into(), name.clone()];
                let (_, _, args) = with_input("Trace", trace);
                lenses.push((line, "Trace".into(), args));
            }
            continue;
        };
        let target = judge.args.first().map(|a| {
            let span = a.value.span;
            text.get(base + span.start as usize..base + span.end as usize)
                .unwrap_or("")
                .trim()
        });
        let alt = matches!(judge.name.text.as_str(), "alt" | "brute");
        let title = match (judge.name.text.as_str(), target) {
            (_, Some("program") | None) if alt => format!("Stress the program against {name}"),
            (_, Some(target)) if alt => format!("Stress against {target}"),
            ("interactor", _) => format!("Stress with interactor {name}"),
            _ => format!("Stress with checker {name}"),
        };
        lenses.push((line, title, vec!["stress".into(), path.into()]));
        if alt {
            let mut trace = vec!["run".into(), path.into(), "--trace".into()];
            if !matches!(target, Some("program") | None) {
                trace.push(name.clone());
            }
            let (_, _, args) = with_input("Trace", trace);
            lenses.push((line, "Trace".into(), args));
        }
    }
    lenses
}

/// The most memory the input bounds let the file's top-level values take,
/// lists only pushed to in counted loops among them,
/// against the memory limit of the judge its build is for. Clicking it
/// lists the judges. None under 1 KB.
fn memory_lens(
    db: &mut cpl_check::Session,
    uri: &str,
    path: &str,
    text: &str,
) -> Option<(String, Vec<String>)> {
    let (mut bytes, values) = db.storage(uri)?;
    let lists = cpl_lint::list_sizes(text);
    let mut unbounded = Vec::new();
    for (name, per) in values {
        match (per, lists.iter().find(|(list, _)| *list == name)) {
            (Some(per), Some((_, most))) => bytes += most * u128::from(per),
            _ => unbounded.push(name),
        }
    }
    if bytes < 1024 {
        return None;
    }
    let judge = cpl_judge::resolve(std::path::Path::new(path), text, None).ok()?;
    let limit = u128::from(judge.memory_mb) << 20;
    let (shown, mb, name) = (cpl_check::human_bytes(bytes), judge.memory_mb, judge.name);
    let mut title = match bytes > limit {
        true => format!("Memory: bounds allow {shown}, over the {mb} MB limit on {name}"),
        false => format!("Memory: bounds allow {shown} of {mb} MB on {name}"),
    };
    match unbounded.as_slice() {
        [] => {}
        [one] => title += &format!(", not counting {one}"),
        [one, rest @ ..] => title += &format!(", not counting {one} and {} more", rest.len()),
    }
    Some((title, vec!["judges".into(), path.into()]))
}

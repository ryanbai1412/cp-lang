//! What a part's `@alt(f)` brute shows about it: the first call of the
//! part that returns something else than its alt on the same arguments.
//! `cpl stress` and the browser's in-process stress both explain with it.

use cpl_interp::trace::{mentions, named_values, Divergence, Recording};

/// With `@alt(f)` fns swapped in, the call of a part `f` that first
/// returns something else than its alt does on the same arguments, with
/// every part it calls right: in a recursion, the innermost wrong call.
pub fn first_call_divergence(
    source: &str,
    fast: &Recording,
    slow: &Recording,
) -> Option<Divergence> {
    let alts = crate::cli::part_alts(source);
    let line_of = |name: &str| {
        let head = format!("fn {name}(");
        (source
            .lines()
            .position(|line| line.trim_start().starts_with(&head)))
        .map_or(0, |at| at as u32 + 1)
    };
    let wrong: Vec<_> = (fast.calls.iter().enumerate())
        .filter_map(|(at, ours)| {
            let (_, alt) = alts.iter().find(|(part, _)| *part == ours.name)?;
            let theirs = (slow.calls.iter())
                .find(|call| call.name == ours.name && call.args == ours.args)?;
            match (&ours.returned, &theirs.returned) {
                (Some(a), Some(b)) if a != b => Some((at, ours, alt, a, b)),
                _ => None,
            }
        })
        .collect();
    // The calls made inside a call follow it, deeper than it: they end
    // where a call no deeper starts.
    let mut ends = vec![fast.calls.len(); fast.calls.len()];
    let mut open: Vec<usize> = Vec::new();
    for (at, call) in fast.calls.iter().enumerate() {
        while let Some(&o) = open.last().filter(|&&o| fast.calls[o].depth >= call.depth) {
            ends[o] = at;
            open.pop();
        }
        open.push(at);
    }
    let &(at, ours, alt, a, b) = wrong.iter().enumerate().find_map(|(k, entry)| {
        let next_wrong = wrong.get(k + 1).map_or(usize::MAX, |(other, ..)| *other);
        (next_wrong >= ends[entry.0]).then_some(entry)
    })?;
    let nested: Vec<_> = (fast.calls[at + 1..ends[at]].iter())
        .map(|call| call.entry..call.exit)
        .collect();
    let mut inside: Vec<(u32, String)> = Vec::new();
    for step in (ours.entry..ours.exit).rev() {
        if inside.len() == 6 {
            break;
        }
        if nested.iter().any(|range| range.contains(&step)) {
            continue;
        }
        let Some(step) = fast.step(step) else {
            continue;
        };
        let shown = (step.line, shown_args(&step.locals));
        if inside.last() != Some(&shown) {
            inside.push(shown);
        }
    }
    inside.reverse();
    let own: Vec<_> = (ours.entry..ours.exit)
        .filter(|step| !nested.iter().any(|range| range.contains(step)))
        .filter_map(|step| fast.step(step))
        .collect();
    let mut blame = leaked(source, fast, ours);
    blame.extend(made(source, fast, ours, &own));
    Some(Divergence {
        inside,
        blame,
        call: true,
        stuck: false,
        name: format!("{}({})", ours.name, shown_args(&ours.args)),
        nth: ours.ordinal,
        fast: Some((line_of(&ours.name), a.clone())),
        brute: Some((line_of(alt), b.clone())),
    })
}

/// A top-level `var` declared outside every fn that a later call of a fn
/// starts with another value than its 1st call did: one test left it for
/// the next. Each such variable's declaration, with both values.
fn leaked(source: &str, rec: &Recording, call: &cpl_interp::trace::Call) -> Vec<(u32, String)> {
    let Some(first) = (rec.calls.iter()).find(|c| c.name == call.name && c.ordinal == 1) else {
        return Vec::new();
    };
    if call.ordinal == 1 {
        return Vec::new();
    }
    let lines: Vec<&str> = source.lines().collect();
    let Some(body) = body_of(&lines, &call.name) else {
        return Vec::new();
    };
    let body = lines[body].join("\n");
    let (Some(then), Some(now)) = (rec.step(first.entry), rec.step(call.entry)) else {
        return same_args(&lines, &body, first, call);
    };
    let mut out = Vec::new();
    for (var, value) in &now.values {
        let name = &rec.vars[*var].name;
        let before = then.values.iter().find(|(v, _)| v == var).map(|(_, v)| v);
        let declared = (lines.iter()).position(|line| {
            line.strip_prefix("var ")
                .is_some_and(|rest| rest.starts_with(name.as_str()) && mentions(rest, name))
        });
        if let (Some(before), Some(at)) = (before, declared) {
            if before != value && mentions(&body, name) {
                out.push((
                    at as u32 + 1,
                    format!(
                        "`{name}` is declared once for the whole run: this call of `{}` starts with `{name}` = {value}, its 1st call with {before}",
                        call.name
                    ),
                ));
            }
        }
    }
    if out.is_empty() {
        return same_args(&lines, &body, first, call);
    }
    out
}

/// A later call of a fn with the same arguments as its 1st call that
/// gives another value reads state the arguments don't carry: the
/// top-level `var`s its body mentions, each declared once for the run.
fn same_args(
    lines: &[&str],
    body: &str,
    first: &cpl_interp::trace::Call,
    call: &cpl_interp::trace::Call,
) -> Vec<(u32, String)> {
    if first.args != call.args || first.returned == call.returned || first.returned.is_none() {
        return Vec::new();
    }
    let word = |c: char| c.is_alphanumeric() || c == '_';
    (lines.iter().enumerate())
        .filter_map(|(at, line)| {
            let rest = line.strip_prefix("var ")?;
            let name = &rest[..rest.find(|c: char| !word(c)).unwrap_or(rest.len())];
            mentions(body, name).then(|| {
                (
                    at as u32 + 1,
                    format!(
                        "`{name}` is declared once for the whole run: call {} of `{}` gets the arguments its 1st call got, {}, and gives {} where that gave {}",
                        call.ordinal,
                        call.name,
                        shown_args(&call.args),
                        call.returned.as_deref().unwrap_or("nothing"),
                        first.returned.as_deref().unwrap_or("nothing"),
                    ),
                )
            })
        })
        .collect()
}

/// The lines of the body of `fn name`, 0-based, after its head.
fn body_of(lines: &[&str], name: &str) -> Option<std::ops::Range<usize>> {
    let head = format!("fn {name}(");
    let at = lines.iter().position(|line| line.starts_with(&head))?;
    let end = (at + 1..lines.len())
        .find(|&k| !lines[k].is_empty() && !lines[k].starts_with(' '))
        .unwrap_or(lines.len());
    let end = (at + 1..end)
        .rev()
        .find(|&k| !lines[k].trim().is_empty())
        .map_or(at + 1, |k| k + 1);
    Some(at..end)
}

/// Where a wrong call made its value: the `return` it ran, else the last
/// line of its body, and the lines that last changed each variable that
/// line reads.
fn made(
    source: &str,
    rec: &Recording,
    call: &cpl_interp::trace::Call,
    own: &[cpl_interp::trace::Step],
) -> Vec<(u32, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let Some(last) = own.last() else {
        return constant_returns(&lines, call);
    };
    let text_of = |line: u32| lines.get(line as usize - 1).copied().unwrap_or("");
    let returned = mentions(text_of(last.line), "return");
    let line = match body_of(&lines, &call.name) {
        Some(body) if !returned && body.len() > 1 => body.end as u32,
        _ => last.line,
    };
    let text = text_of(line);
    let mut out = vec![(line, format!("`{}` makes its value here", call.name))];
    let before = (call.entry.checked_sub(1))
        .and_then(|step| rec.step(step))
        .map(|step| named_values(rec, &step))
        .unwrap_or_default();
    let steps: Vec<_> = own
        .iter()
        .map(|step| (step.line, named_values(rec, step)))
        .collect();
    let mut names: Vec<&String> = (steps.last().map(|(_, values)| values.keys()))
        .into_iter()
        .flatten()
        .filter(|name| mentions(text, name))
        .collect();
    names.sort();
    // The last step before `upto` that changed `name`: its index, line and value.
    let last_change = |name: &str, upto: usize| {
        let mut prev = before.get(name);
        let mut at = None;
        for (k, (line, values)) in steps[..upto].iter().enumerate() {
            let now = values.get(name);
            if let Some(value) = now.filter(|_| now != prev) {
                at = Some((k, *line, value.clone()));
            }
            prev = now;
        }
        at
    };
    let mut changed = Vec::new();
    let mut stale = Vec::new();
    let mut reads = Vec::new();
    let mut guarded = Vec::new();
    for name in names {
        let at = last_change(name, steps.len());
        if let Some((k, at, value)) = at.filter(|(_, at, _)| *at != line) {
            // One step further back: what that line reads, where it last changed.
            let mut read: Vec<&String> = (steps[k].1.keys())
                .filter(|n| *n != name && mentions(text_of(at), n))
                .collect();
            read.sort();
            for g in guards(&lines, at) {
                let mut checked: Vec<&String> = (steps[k].1.keys())
                    .filter(|n| *n != name && mentions(text_of(g), n))
                    .collect();
                checked.sort();
                for n in checked {
                    let Some((_, from, _)) =
                        last_change(n, k).filter(|(_, l, _)| ![at, line, g].contains(l))
                    else {
                        continue;
                    };
                    guarded.push((
                        from,
                        format!(
                            "`{n}`, which line {g} checks before line {at} runs, last changed here"
                        ),
                    ));
                    if let Some(cond) = guards(&lines, from).into_iter().find(|&c| c < from) {
                        guarded.push((cond, format!("line {from} runs only when this holds")));
                    }
                }
            }
            for n in read {
                if let Some((_, from, v)) =
                    last_change(n, k).filter(|(_, l, _)| ![at, line].contains(l))
                {
                    reads.push((
                        from,
                        format!("`{n}`, which line {at} reads, last changed here, to {v}"),
                    ));
                    reads.extend(ordered_by(&lines, &call.name, from));
                }
            }
            changed.push((at, format!("`{name}` last changed here, to {value}")));
            // Still as declared: the lines that write it never changed it.
            if text_of(at).trim_start().starts_with(&format!("var {name}")) {
                stale.extend((body_of(&lines, &call.name).into_iter().flatten())
                    .map(|k| k as u32 + 1)
                    .filter(|&k| k != at && writes(text_of(k), name))
                    .take(2)
                    .map(|k| (k, format!("`{name}` is still as line {at} declared it: this line is the one that would change it"))));
            }
        }
    }
    changed.sort();
    reads.sort();
    out.extend(changed.into_iter().take(3));
    out.extend(stale);
    out.extend(reads.into_iter().take(2));
    out.extend(guarded.into_iter().take(2));
    let mut seen = std::collections::HashSet::new();
    out.retain(|(line, _)| seen.insert(*line));
    out
}

/// Where the container line `at` takes an end of (`heap.pop()`,
/// `set.first`) is declared, when its kind decides which end that is.
fn ordered_by(lines: &[&str], fn_name: &str, at: u32) -> Option<(u32, String)> {
    let text = lines.get(at as usize - 1)?;
    let body = body_of(lines, fn_name)?;
    let word = |c: char| c.is_alphanumeric() || c == '_';
    ["pop()", "top", "peek", "first", "last", "min", "max"]
        .iter()
        .find_map(|end| {
            let (before, _) = text.split_once(&format!(".{end}"))?;
            let name = &before[before.rfind(|c: char| !word(c)).map_or(0, |k| k + 1)..];
            let k = body.clone().find(|&k| {
                let head = lines[k].trim_start();
                (head.strip_prefix("var ").or_else(|| head.strip_prefix("let ")))
                    .and_then(|rest| rest.strip_prefix(name))
                    .is_some_and(|rest| rest.starts_with([' ', ':', '=']))
            })?;
            let kind = ["MinHeap", "MaxHeap", "Heap", "Set", "Map"]
                .into_iter()
                .find(|kind| lines[k].contains(kind))?;
            (!name.is_empty()).then(|| {
                (
                    k as u32 + 1,
                    format!("`{name}.{end}` at line {at} takes the end that the {kind} declared here keeps"),
                )
            })
        })
}

/// The 1-based lines whose test decides whether line `at` runs: the
/// `if` it is nested in and the `if ...: continue` (or `break`, `return`)
/// lines before it in its loop, nearest first.
fn guards(lines: &[&str], at: u32) -> Vec<u32> {
    let indent = |text: &str| text.len() - text.trim_start().len();
    let Some(mut depth) = lines.get(at as usize - 1).map(|text| indent(text)) else {
        return Vec::new();
    };
    let mut out = Vec::new();
    for k in (0..at as usize - 1).rev() {
        let text = lines[k];
        let head = text.trim_start();
        if head.is_empty() || head.starts_with("//") {
            continue;
        }
        if indent(text) == depth
            && head.starts_with("if ")
            && [": continue", ": break", ": return"]
                .iter()
                .any(|exit| head.contains(exit))
        {
            out.push(k as u32 + 1);
        } else if indent(text) < depth {
            if !(head.starts_with("if ") || head.starts_with("elif ")) {
                break;
            }
            out.push(k as u32 + 1);
            depth = indent(text);
        }
    }
    out
}

/// Whether `text` sets `name` or one of its items: `name = ...`,
/// `name[i] += ...`, never `name == ...`.
fn writes(text: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(name).any(|(at, _)| {
        if text[..at].ends_with(word) {
            return false;
        }
        let mut rest = &text[at + name.len()..];
        if rest.starts_with(word) {
            return false;
        }
        if rest.starts_with('[') {
            let mut depth = 0;
            let Some(end) = rest.find(|c| {
                depth += match c {
                    '[' => 1,
                    ']' => -1,
                    _ => 0,
                };
                depth == 0
            }) else {
                return false;
            };
            rest = &rest[end + 1..];
        }
        let rest = rest.trim_start();
        let op = rest.find('=').unwrap_or(rest.len());
        let ops = &rest[..op];
        op < rest.len()
            && !rest[op + 1..].starts_with('=')
            && (ops.chars().all(|c| "+-*/%&|^<>".contains(c)) || matches!(ops, "max" | "min"))
    })
}

/// With no step of a wrong call recorded, the `return`s in its body of
/// the very value it gave: the only lines that make that value as it is.
fn constant_returns(lines: &[&str], call: &cpl_interp::trace::Call) -> Vec<(u32, String)> {
    let (Some(body), Some(value)) = (body_of(lines, &call.name), &call.returned) else {
        return Vec::new();
    };
    (body.clone().zip(&lines[body]))
        .filter(|(_, text)| {
            let ret = text
                .find("return ")
                .map(|at| text[at + 7..].split("//").next().unwrap_or(""));
            ret.is_some_and(|ret| ret.trim() == value)
        })
        .map(|(at, _)| {
            (
                at as u32 + 1,
                format!("`{}` returns {value} here", call.name),
            )
        })
        .collect()
}

/// A call's arguments, JSON by name, as `k: 1, a: [2, 1]`.
fn shown_args(json: &str) -> String {
    match cpl_json::parse(json) {
        Ok(cpl_json::Value::Object(args)) => (args.iter())
            .map(|(name, value)| format!("{name}: {}", cpl_json::to_string(value)))
            .collect::<Vec<_>>()
            .join(", "),
        _ => json.to_string(),
    }
}

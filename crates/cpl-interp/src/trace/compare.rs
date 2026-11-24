//! Two runs of a file side by side: where they first part ways.

use super::{string, Assigned, Recording, Shown};
use std::collections::HashMap;

/// A checkpoint both runs reached, with what each showed there.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Stage {
    pub name: String,
    /// 1-based line in the solution.
    pub line: u32,
    pub fast: String,
    pub brute: String,
}

/// Where a solution and its brute first give a variable they both name
/// different values: its `nth` value (1-based, a repeat of the value
/// before not counted), with the line and the value on each side, `None`
/// on a side that never gives it an `nth` value.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Divergence {
    pub name: String,
    pub nth: usize,
    pub fast: Option<(u32, String)>,
    pub brute: Option<(u32, String)>,
    /// The solution's own last steps in a wrong call, not those of the
    /// calls it makes: each line and its locals, as `k: 1, s: 0`.
    pub inside: Vec<(u32, String)>,
    /// The lines to look at, each with why: where a wrong call made its
    /// value, where what that line reads last changed, or the declaration
    /// of a variable an earlier test left a value in.
    pub blame: Vec<(u32, String)>,
    /// `name` is a call, `f(k: 1)`, and `nth` which call of `f` it is.
    pub call: bool,
    /// The solution never finishes: `name` is the loop it goes round in,
    /// `fast` that loop's line and the state it keeps coming back to, and
    /// `nth` how many steps one time round takes.
    pub stuck: bool,
}

/// The first value the solution gives one of the variables both runs
/// name that the brute does not give it, from [`Recording::assigned`]. A
/// variable that takes the same values in another order and ends alike
/// (a loop walked the other way) is not a divergence.
pub fn first_divergence(fast: &Recording, slow: &Recording) -> Option<Divergence> {
    fn histories(rec: &Recording) -> HashMap<&str, Vec<(usize, &Assigned)>> {
        let mut out: HashMap<&str, Vec<(usize, &Assigned)>> = HashMap::new();
        for (at, assigned) in rec.assigned.iter().enumerate() {
            let history = out.entry(assigned.name.as_str()).or_default();
            if history
                .last()
                .is_none_or(|(_, last)| last.value != assigned.value)
            {
                history.push((at, assigned));
            }
        }
        out
    }
    let (ours, theirs) = (histories(fast), histories(slow));
    let mut best: Option<(usize, &str, usize)> = None;
    for (name, a) in &ours {
        let Some(b) = theirs.get(name) else {
            continue;
        };
        let values = |history: &[(usize, &Assigned)]| -> Vec<String> {
            history
                .iter()
                .map(|(_, assigned)| assigned.value.clone())
                .collect()
        };
        let (va, vb) = (values(a), values(b));
        if va == vb {
            continue;
        }
        let (mut sa, mut sb) = (va.clone(), vb.clone());
        sa.sort();
        sb.sort();
        if sa == sb && va.last() == vb.last() {
            continue;
        }
        let nth = va.iter().zip(&vb).take_while(|(x, y)| x == y).count();
        let at = a.get(nth).map_or(fast.assigned.len(), |(at, _)| *at);
        if best.is_none_or(|(best_at, best_name, _)| (at, *name) < (best_at, best_name)) {
            best = Some((at, name, nth));
        }
    }
    let (_, name, nth) = best?;
    let side = |history: &Vec<(usize, &Assigned)>| {
        history
            .get(nth)
            .map(|(_, assigned)| (assigned.line, assigned.value.clone()))
    };
    Some(Divergence {
        name: name.to_string(),
        nth: nth + 1,
        fast: side(&ours[name]),
        brute: side(&theirs[name]),
        inside: Vec::new(),
        blame: Vec::new(),
        call: false,
        stuck: false,
    })
}

/// The `debug(...)` checkpoints both runs reached, in the solution's
/// order, and the last line each printed: where the two first differ is
/// where to look.
pub fn stages(fast: &Recording, slow: &Recording) -> Vec<Stage> {
    let theirs: HashMap<&str, &Shown> = slow
        .shown
        .iter()
        .map(|shown| (shown.name.as_str(), shown))
        .collect();
    fast.shown
        .iter()
        .filter_map(|shown| {
            let other = theirs.get(shown.name.as_str())?;
            Some(Stage {
                name: shown.name.clone(),
                line: shown.line,
                fast: shown.text.clone(),
                brute: other.text.clone(),
            })
        })
        .collect()
}

/// One run of a trace document: `{"label": .., "steps": [..]}`.
pub(super) fn run_json(
    label: &str,
    rec: &Recording,
    ids: &[String],
    dropped: &dyn Fn(&str) -> bool,
) -> String {
    let mut out = String::from("{\"label\":");
    string(&mut out, label);
    if rec.skipped > 0 {
        out += &format!(",\"firstStep\":{}", rec.skipped);
    }
    if rec.truncated {
        out.push_str(",\"truncated\":true");
    }
    out.push_str(",\"steps\":[");
    for (index, step) in rec.steps(0..rec.len()).enumerate() {
        if index > 0 {
            out.push(',');
        }
        out += &format!("{{\"id\":{index},\"line\":{},\"label\":", step.line);
        string(&mut out, &step.label);
        out.push_str(",\"stage\":");
        string(&mut out, &step.stage);
        out.push_str(",\"values\":{");
        for (i, (var, json)) in step.values.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            string(&mut out, &ids[*var]);
            out.push(':');
            out.push_str(json);
        }
        out.push_str("},\"locals\":");
        out.push_str(&step.locals);
        if let Some(checkpoint) = step.checkpoint.as_deref().filter(|name| !dropped(name)) {
            out.push_str(",\"checkpoint\":");
            string(&mut out, checkpoint);
        }
        if !step.frames.is_empty() {
            out.push_str(",\"frames\":[");
            for (i, (name, line)) in step.frames.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str("{\"fn\":");
                string(&mut out, name);
                out += &format!(",\"line\":{line}}}");
            }
            out.push(']');
        }
        if let Some((var, path)) = &step.focus {
            out.push_str(",\"focus\":{\"variable\":");
            string(&mut out, &ids[*var]);
            let path: Vec<String> = path.iter().map(usize::to_string).collect();
            out += &format!(",\"path\":[{}]}}", path.join(","));
        }
        out.push('}');
    }
    out.push_str("]}");
    out
}

/// With no variable both runs name going wrong, where the solution's output
/// first leaves the brute's: the `print` that wrote the first wrong line,
/// the lines that last changed what it prints, and the loop each sits in.
pub fn printed_divergence(source: &str, fast: &Recording, slow: &Recording) -> Option<Divergence> {
    let lines: Vec<&str> = source.lines().collect();
    let theirs = slow
        .step(slow.len().checked_sub(1)?)
        .map(|s| printed(slow, &s))?;
    let steps: Vec<_> = fast.steps(0..fast.len()).collect();
    let wrong = |out: &[String]| {
        out.iter()
            .enumerate()
            .position(|(k, ours)| match theirs.get(k) {
                Some(their) if k + 1 == out.len() => !their.starts_with(ours.as_str()),
                Some(their) => their != ours,
                None => true,
            })
    };
    let (at, k, ours) = steps.iter().enumerate().find_map(|(at, step)| {
        let out = printed(fast, step);
        let k = wrong(&out)?;
        Some((at, k, out[k].clone()))
    })?;
    let print = steps[at].line;
    let blame = output_trail(&lines, fast, &steps, at);
    let their_line = (slow.steps(0..slow.len()))
        .find(|step| printed(slow, step).len() > k)
        .map_or(0, |step| step.line);
    Some(Divergence {
        name: "output".to_string(),
        nth: k + 1,
        fast: Some((print, ours)),
        brute: Some((their_line, theirs[k].clone())),
        inside: Vec::new(),
        blame,
        call: false,
        stuck: false,
    })
}

/// The answer a judge rejected: the last line `fast` wrote, the `print`
/// that wrote it, and the lines that last changed what it shows.
pub fn answered_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
    let lines: Vec<&str> = source.lines().collect();
    let steps: Vec<_> = fast.steps(0..fast.len()).collect();
    let total = printed(fast, steps.last()?).len();
    let at = (steps.iter()).position(|step| printed(fast, step).len() == total)?;
    let ours = printed(fast, &steps[at]).pop()?;
    Some(Divergence {
        name: "answer".to_string(),
        nth: total,
        fast: Some((steps[at].line, ours)),
        brute: None,
        inside: Vec::new(),
        blame: output_trail(&lines, fast, &steps, at),
        call: false,
        stuck: false,
    })
}

/// The lines that last set what the `print` at step `at` shows, and the
/// loops they run in.
fn output_trail(
    lines: &[&str],
    fast: &Recording,
    steps: &[super::Step],
    at: usize,
) -> Vec<(u32, String)> {
    let text_of = |line: u32| {
        lines
            .get((line as usize).wrapping_sub(1))
            .copied()
            .unwrap_or("")
    };
    let print = steps[at].line;
    let values: Vec<HashMap<String, String>> = steps[..=at]
        .iter()
        .map(|step| named_values(fast, step))
        .collect();
    let mut names: Vec<&String> = (values[at].keys())
        .filter(|name| *name != "stdout" && mentions(text_of(print), name))
        .collect();
    names.sort();
    let mut blame: Vec<(u32, String)> = Vec::new();
    for name in names {
        let set = (0..=at).rev().find(|&s| {
            steps[s].line != print && assigned_name(text_of(steps[s].line)) == Some(name.as_str())
        });
        let changed = (0..=at).rev().find(|&s| {
            let now = values[s].get(name);
            now.is_some() && (s == 0 || values[s - 1].get(name) != now)
        });
        let whys = match (set, changed) {
            (Some(s), Some(c)) if s > c => {
                vec![(s, "is last set here, and stays {v}"), (c, "gets {v} here")]
            }
            (_, Some(c)) => vec![(c, "last changed here, to {v}")],
            (Some(s), None) => vec![(s, "is last set here, to {v}")],
            (None, None) => Vec::new(),
        };
        for (s, why) in whys.into_iter().filter(|&(s, _)| steps[s].line != print) {
            let value = &values[s][name];
            blame.push((
                steps[s].line,
                format!(
                    "`{name}`, which line {print} prints, {}",
                    why.replace("{v}", value)
                ),
            ));
        }
        blame.extend(unrun_sets(lines, fast, print, name));
    }
    blame.sort();
    blame.truncate(3);
    let loops: Vec<(u32, String)> = (blame.iter())
        .filter_map(|(line, _)| {
            let indent = |text: &str| text.len() - text.trim_start().len();
            let own = indent(text_of(*line));
            let head = (1..*line).rev().find(|&l| {
                let text = text_of(l);
                !text.trim().is_empty() && indent(text) < own
            })?;
            let text = text_of(head).trim_start();
            (text.starts_with("for ") || text.starts_with("while "))
                .then(|| (head, format!("line {line} runs in this loop")))
        })
        .collect();
    blame.extend(loops);
    blame.dedup_by_key(|(line, _)| *line);
    blame
}

/// The loops and `if`s that kept a line setting `name` from running at
/// all: `for i in 1..n` running 0 times leaves `best` at its start.
fn unrun_sets(lines: &[&str], fast: &Recording, print: u32, name: &str) -> Vec<(u32, String)> {
    if fast.assigned.len() >= super::MAX_ASSIGNED {
        return Vec::new();
    }
    let indent = |text: &str| text.len() - text.trim_start().len();
    let head = |line: usize| {
        let own = indent(lines[line]);
        (0..line)
            .rev()
            .find(|&l| !lines[l].trim().is_empty() && indent(lines[l]) < own)
    };
    let mut blame = Vec::new();
    for (at, text) in lines.iter().enumerate() {
        let line = at as u32 + 1;
        let ran = || (fast.assigned.iter()).any(|a| a.line == line && a.name == name);
        if line == print || assigned_name(text) != Some(name) || ran() {
            continue;
        }
        let Some(guard) = head(at) else { continue };
        let mut outer = Some(guard);
        let in_fn = std::iter::from_fn(|| {
            let l = outer?;
            outer = head(l);
            Some(lines[l].trim_start())
        })
        .any(|text| text.starts_with("fn ") || text.starts_with('@'));
        let guard_text = lines[guard].trim_start();
        let why = if guard_text.starts_with("for ") || guard_text.starts_with("while ") {
            "in this loop, which runs 0 times"
        } else if guard_text.starts_with("if ") {
            "under this `if`, which is false every time"
        } else {
            continue;
        };
        if !in_fn {
            blame.push((
                guard as u32 + 1,
                format!("`{name}` is set on line {line} only {why}"),
            ));
        }
    }
    blame
}

/// What a run had printed by `step`, a line at a time.
fn printed(rec: &Recording, step: &super::Step) -> Vec<String> {
    let at = rec.vars.iter().position(|var| var.name == "stdout");
    let json = (step.values.iter()).find(|(var, _)| Some(*var) == at);
    match json.map(|(_, json)| cpl_json::parse(json)) {
        Some(Ok(cpl_json::Value::Array(lines))) => (lines.into_iter())
            .map(|line| match line {
                cpl_json::Value::String(text) => text,
                other => cpl_json::to_string(&other),
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Each variable a step shows, top-level and local, by name, as JSON.
pub fn named_values(rec: &Recording, step: &super::Step) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = (step.values.iter())
        .map(|(var, value)| (rec.vars[*var].name.clone(), value.to_string()))
        .collect();
    if let Ok(cpl_json::Value::Object(locals)) = cpl_json::parse(&step.locals) {
        for (name, value) in locals {
            out.insert(name, cpl_json::to_string(&value));
        }
    }
    out
}

/// Whether `name` is a word of `text`.
pub fn mentions(text: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    text.match_indices(name)
        .any(|(at, _)| !text[..at].ends_with(word) && !text[at + name.len()..].starts_with(word))
}

/// Where a solution that never finishes goes round without getting
/// anywhere: the first step whose line and every value repeat an earlier
/// step's, the innermost loop around that cycle, and the assignments in it
/// that leave their variable as they found it.
pub fn stuck_divergence(source: &str, fast: &Recording) -> Option<Divergence> {
    let lines: Vec<&str> = source.lines().collect();
    let text_of = |line: u32| {
        lines
            .get((line as usize).wrapping_sub(1))
            .copied()
            .unwrap_or("")
    };
    let steps: Vec<_> = fast.steps(0..fast.len()).collect();
    let mut states: Vec<HashMap<String, String>> = Vec::new();
    let mut seen: HashMap<(u32, Vec<(String, String)>), usize> = HashMap::new();
    let mut cycle = None;
    for (at, step) in steps.iter().enumerate() {
        let state = named_values(fast, step);
        let mut key: Vec<_> = state.clone().into_iter().collect();
        key.sort();
        states.push(state);
        if let Some(before) = seen.insert((step.line, key), at) {
            cycle = Some((before, at));
            break;
        }
    }
    let (from, to) = cycle?;
    let round: Vec<u32> = steps[from + 1..=to].iter().map(|step| step.line).collect();
    let indent = |text: &str| text.len() - text.trim_start().len();
    let is_loop = |line: u32| {
        let text = text_of(line).trim_start();
        text.starts_with("for ") || text.starts_with("while ")
    };
    let loops_around = |line: u32| -> Vec<u32> {
        let mut out: Vec<u32> = Vec::new();
        let mut own = indent(text_of(line));
        if is_loop(line) {
            out.push(line);
        }
        for l in (1..line).rev() {
            let text = text_of(l);
            if !text.trim().is_empty() && indent(text) < own {
                own = indent(text);
                if is_loop(l) {
                    out.push(l);
                }
            }
        }
        out
    };
    let head = (loops_around(round[0]).into_iter())
        .find(|head| round.iter().all(|&line| loops_around(line).contains(head)))?;
    let state = &states[to];
    let mut held: Vec<String> = (state.iter())
        .filter(|(name, _)| *name != "stdout" && mentions(text_of(head), name))
        .map(|(name, value)| format!("{name} = {value}"))
        .collect();
    held.sort();
    let mut blame: Vec<(u32, String)> = Vec::new();
    for at in from + 1..=to {
        let line = steps[at].line;
        let Some(name) = assigned_name(text_of(line)) else {
            continue;
        };
        let (before, after) = (states[at - 1].get(name), states[at].get(name));
        if let Some(value) = after.filter(|_| before == after) {
            let why = format!("`{name}` is set to {value} here, the value it already had");
            if !blame.iter().any(|(l, _)| *l == line) {
                blame.push((line, why));
            }
        }
    }
    Some(Divergence {
        name: text_of(head).trim().trim_end_matches(':').to_string(),
        nth: to - from,
        fast: Some((head, held.join(", "))),
        brute: None,
        inside: Vec::new(),
        blame,
        call: false,
        stuck: true,
    })
}

/// The variable a line of the form `name = ...` or `name += ...` sets.
fn assigned_name(text: &str) -> Option<&str> {
    let text = text.trim_start();
    let end = text.find(|c: char| !(c.is_alphanumeric() || c == '_'))?;
    let rest = text[end..].trim_start();
    let op = rest.find('=')?;
    let ops = &rest[..op];
    let op_ok = ops.chars().all(|c| "+-*/%&|^<>".contains(c)) || matches!(ops, "max" | "min");
    let set = !rest[op + 1..].starts_with('=') && op_ok;
    (end > 0 && set && !matches!(&text[..end], "let" | "var" | "if" | "while" | "for"))
        .then(|| &text[..end])
}

/// What `cpl stress --explain` reports when no call went wrong: the first
/// value the solution gives differently, with where its first wrong output
/// came from, else that output alone.
pub fn explain_divergence(source: &str, fast: &Recording, slow: &Recording) -> Option<Divergence> {
    let printed = || printed_divergence(source, fast, slow);
    match first_divergence(fast, slow) {
        Some(mut d) if d.blame.is_empty() => {
            d.blame = printed().map(|p| p.blame).unwrap_or_default();
            Some(d)
        }
        Some(d) => Some(d),
        None => printed(),
    }
}

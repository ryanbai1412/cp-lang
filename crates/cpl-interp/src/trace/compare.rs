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

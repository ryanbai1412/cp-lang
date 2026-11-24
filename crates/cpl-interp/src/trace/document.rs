//! The trace document the viewer reads.

use super::*;

/// The trace document of one run, or of a solution (`fast`) and its
/// brute (`slow`) on the same input. The two are lined up on the
/// checkpoints they share, and compared on what both printed and on the
/// `debug(...)`s both ran.
pub fn document(
    title: &str,
    description: &str,
    file: &str,
    source: &str,
    fast: (&str, &Recording),
    slow: Option<(&str, &Recording)>,
) -> String {
    let (fast_label, fast) = fast;
    // One table of variables for both runs: the same name is the same
    // variable when both describe it alike.
    let mut vars: Vec<Var> = fast.vars.clone();
    let fast_ids: Vec<String> = fast.vars.iter().map(|var| var.id.clone()).collect();
    let mut slow_ids = Vec::new();
    let mut dropped = std::collections::HashSet::new();
    if let Some((_, slow)) = slow {
        for var in &slow.vars {
            let same = vars.iter().find(|known| {
                known.id == var.id && known.kind == var.kind && known.extra == var.extra
            });
            match same {
                Some(_) => slow_ids.push(var.id.clone()),
                None if vars.iter().any(|known| known.id == var.id) => {
                    let mut var = var.clone();
                    var.id = format!("{} (brute)", var.id);
                    slow_ids.push(var.id.clone());
                    vars.push(var);
                }
                None => {
                    slow_ids.push(var.id.clone());
                    vars.push(var.clone());
                }
            }
        }
        // Shared checkpoints must come in one order in both runs: one
        // that does not is left as a plain step.
        let position: HashMap<&str, usize> =
            slow.checkpoints().map(|(i, name)| (name, i)).collect();
        let mut reached = None;
        for (_, name) in fast.checkpoints() {
            match position.get(name) {
                Some(at) if reached.is_none_or(|last| *at > last) => reached = Some(*at),
                Some(_) => {
                    dropped.insert(name.to_string());
                }
                None => {}
            }
        }
    }
    let mut out = String::from("{\"schemaVersion\":1,\"title\":");
    string(&mut out, title);
    out.push_str(",\"description\":");
    string(&mut out, description);
    out.push_str(",\"source\":{\"file\":");
    string(&mut out, file);
    out.push_str(",\"code\":");
    string(&mut out, source);
    out.push_str("},\"variables\":{");
    for (i, var) in vars.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        string(&mut out, &var.id);
        out.push_str(":{\"name\":");
        string(&mut out, &var.name);
        out.push_str(",\"type\":");
        string(&mut out, &var.ty);
        out += &format!(",\"kind\":\"{}\"{}}}", var.kind, var.extra);
    }
    let is_dropped = |name: &str| dropped.contains(name);
    out.push_str("},\"runs\":{\"fast\":");
    out += &compare::run_json(fast_label, fast, &fast_ids, &is_dropped);
    if let Some((slow_label, slow)) = slow {
        out.push_str(",\"slow\":");
        out += &compare::run_json(slow_label, slow, &slow_ids, &is_dropped);
        // Compared: what both runs have and mean alike.
        let compare: Vec<&Var> = vars
            .iter()
            .filter(|var| var.compare && fast_ids.contains(&var.id) && slow_ids.contains(&var.id))
            .collect();
        out.push_str("},\"compare\":[");
        for (i, var) in compare.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            string(&mut out, &var.id);
        }
        out.push(']');
    } else {
        out.push('}');
    }
    out.push('}');
    out
}

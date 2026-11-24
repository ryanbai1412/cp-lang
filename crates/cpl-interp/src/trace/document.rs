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
            let same = vars.iter().position(|known| {
                known.id == var.id && known.kind == var.kind && known.extra == var.extra
            });
            match same {
                Some(known) => {
                    vars[known].window |= var.window;
                    slow_ids.push(var.id.clone());
                }
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
    let is_dropped = |name: &str| dropped.contains(name);
    cpl_json::write_with(|w| {
        w.object().key("schemaVersion").int(1);
        w.key("title")
            .str(title)
            .key("description")
            .str(description);
        w.key("source")
            .object()
            .key("file")
            .str(file)
            .key("code")
            .str(source)
            .end();
        w.key("variables").object();
        for var in &vars {
            w.key(&var.id).object();
            w.key("name")
                .str(&var.name)
                .key("type")
                .str(&var.ty)
                .key("kind")
                .str(var.kind);
            for (key, value) in &var.extra {
                w.key(key).value(value);
            }
            if var.window {
                w.key("window").bool(true);
            }
            w.end();
        }
        w.end().key("runs").object().key("fast");
        compare::run_json(w, fast_label, fast, &fast_ids, &is_dropped);
        if let Some((slow_label, slow)) = slow {
            w.key("slow");
            compare::run_json(w, slow_label, slow, &slow_ids, &is_dropped);
            w.end();
            // Compared: what both runs have and mean alike.
            w.key("compare").array();
            for var in &vars {
                if var.compare && fast_ids.contains(&var.id) && slow_ids.contains(&var.id) {
                    w.str(&var.id);
                }
            }
            w.end();
        } else {
            w.end();
        }
        w.end();
    })
}

//! A trap's locals, from a second run recorded to its last step.

use super::{Recording, TraceConfig};

/// How to record a run again to name the locals where it trapped: only
/// its last step is kept.
pub fn trap_config(source: &str) -> TraceConfig {
    TraceConfig {
        source: source.to_string(),
        last_steps: Some(1),
        record_frames: true,
        ..TraceConfig::default()
    }
}

/// The most characters of one local a trap's note shows.
const TRAP_LOCAL_CHARS: usize = 60;
/// The most locals a trap's note names.
const TRAP_LOCALS: usize = 8;

/// The innermost call's locals at the last step of `recording`, as the
/// source writes them: `i = 3, best = [0, 1]`; `None` when it has none.
/// Top-level values the trapping code `at` names are added, and those it
/// names come first. On an unindented top-level line, the block locals of
/// the top level have ended, so they are left out.
pub fn trap_locals(recording: &Recording, source: &str, at: &str) -> Option<String> {
    let step = recording.step(recording.len().checked_sub(1)?)?;
    let named = |name: &str| {
        at.split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|word| word == name)
    };
    let top_level = step.frames.len() == 1;
    let unindented = (step.line as usize)
        .checked_sub(1)
        .and_then(|line| source.lines().nth(line))
        .is_some_and(|line| !line.starts_with([' ', '\t']));
    let mut locals: Vec<(String, cpl_json::Value)> = match cpl_json::parse(&step.locals) {
        Ok(cpl_json::Value::Object(map)) if !(top_level && unindented) => map
            .iter()
            .map(|(name, value)| (name.to_string(), value.clone()))
            .collect(),
        _ => Vec::new(),
    };
    for (var, json) in &step.values {
        let Some(name) = recording.vars.get(*var).map(|v| v.name.as_str()) else {
            continue;
        };
        if named(name) && !locals.iter().any(|(local, _)| local == name) {
            if let Ok(value) = cpl_json::parse(json) {
                locals.push((name.to_string(), value));
            }
        }
    }
    locals.sort_by_key(|(name, _)| !named(name));
    let more = locals.len() > TRAP_LOCALS;
    let mut parts: Vec<String> = locals
        .into_iter()
        .take(TRAP_LOCALS)
        .map(|(name, value)| {
            let text = shown(&value);
            if text.chars().count() > TRAP_LOCAL_CHARS {
                let cut: String = text.chars().take(TRAP_LOCAL_CHARS).collect();
                format!("{name} = {cut}...")
            } else {
                format!("{name} = {text}")
            }
        })
        .collect();
    if more {
        parts.push("...".to_string());
    }
    (!parts.is_empty()).then(|| parts.join(", "))
}

/// A recorded value as the source would write it: `(w: 1, v: 1)`, `None`.
pub fn shown(json: &cpl_json::Value) -> String {
    use cpl_json::Value as Json;
    match json {
        Json::Null => "None".into(),
        Json::String(s) => s.clone(),
        Json::Array(items) => format!(
            "[{}]",
            items.iter().map(shown).collect::<Vec<_>>().join(", ")
        ),
        Json::Object(map) => {
            format!(
                "({})",
                map.iter()
                    .map(|(k, v)| format!("{k}: {}", shown(v)))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
        other => other.to_string(),
    }
}

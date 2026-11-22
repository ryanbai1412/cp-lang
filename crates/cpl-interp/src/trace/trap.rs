//! A trap's locals, from a second run recorded to its last step.

use super::{Recording, TraceConfig};

/// How to record a run again to name the locals where it trapped: only
/// its last step is kept.
pub fn trap_config(source: &str) -> TraceConfig {
    TraceConfig {
        source: source.to_string(),
        last_steps: Some(1),
        ..TraceConfig::default()
    }
}

/// The most characters of one local a trap's note shows.
const TRAP_LOCAL_CHARS: usize = 60;
/// The most locals a trap's note names.
const TRAP_LOCALS: usize = 8;

/// The innermost call's locals at the last step of `recording`, as the
/// source writes them: `i = 3, best = [0, 1]`; `None` when it has none.
pub fn trap_locals(recording: &Recording) -> Option<String> {
    let step = recording.step(recording.len().checked_sub(1)?)?;
    let cpl_json::Value::Object(map) = cpl_json::parse(&step.locals).ok()? else {
        return None;
    };
    let mut parts: Vec<String> = map
        .iter()
        .take(TRAP_LOCALS)
        .map(|(name, value)| {
            let text = shown(value);
            if text.chars().count() > TRAP_LOCAL_CHARS {
                let cut: String = text.chars().take(TRAP_LOCAL_CHARS).collect();
                format!("{name} = {cut}...")
            } else {
                format!("{name} = {text}")
            }
        })
        .collect();
    if map.len() > TRAP_LOCALS {
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

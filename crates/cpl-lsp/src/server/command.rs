//! How an editor runs `cpl` for a lens or a command the user typed, so the
//! editor only forwards and draws: `cpl/command` and `cpl/commands`.

use cpl_json::{obj, Value};

/// The commands whose report an editor draws from its `view` (a panel and
/// the first wrong call on its line) rather than in a terminal.
const VIEWS: &[&str] = &["stress", "locate"];

/// `cpl/command`: the arguments after `cpl` that run `arguments` (none is
/// `test`), with `file` after the command when there is one, and `view`
/// when they ask for JSON whose `view` the editor draws.
pub(super) fn command(params: &Value) -> Value {
    let given = params.get("arguments").and_then(Value::as_array);
    let mut argv: Vec<String> = (given.unwrap_or(&[]).iter())
        .filter_map(Value::as_str)
        .map(String::from)
        .collect();
    if argv.is_empty() {
        argv.push("test".into());
    }
    let file = params.get("file").and_then(Value::as_str);
    if let Some(file) = file.filter(|f| !f.is_empty()) {
        argv.insert(1, file.to_string());
    }
    let view = VIEWS.contains(&argv[0].as_str());
    if view {
        argv.push("--json".into());
    }
    let argv = argv.into_iter().map(Value::from).collect::<Vec<_>>();
    obj([("argv", argv.into()), ("view", view.into())])
}

/// `cpl/commands`: the command names that complete the word after the
/// editor's own command in `line` (`Cpl sub` to `submit`); none past it.
pub(super) fn commands(names: &[&str], params: &Value) -> Value {
    let line = params.get("line").and_then(Value::as_str).unwrap_or("");
    let mut words: Vec<&str> = line.split_whitespace().skip(1).collect();
    if line.ends_with(char::is_whitespace) {
        words.push("");
    }
    let found = match words[..] {
        [lead] => names.iter().filter(|n| n.starts_with(lead)).collect(),
        _ => Vec::new(),
    };
    Value::Array(found.into_iter().map(|n| Value::from(*n)).collect())
}

//! `// tool: dap <step>...` cases: launch the case in `cpl dap` and print
//! every response and event it sends, one JSON message per line, for
//! `// has:` / `// lacks:` to check. Steps: `break:N` (a breakpoint on line
//! N, counted from after the case's `//` header lines), `continue`, `next`, `stepIn`, `stepOut`, `stepBack`,
//! `reverseContinue` and `eval:EXPR`. The case's `// input:` is the
//! program's input and its path prints as `<case>`.

use crate::conformance::ToolRun;
use crate::lsp::frames;
use cpl_json::{obj, Value};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

const STEPS: [&str; 6] = [
    "continue",
    "next",
    "stepIn",
    "stepOut",
    "stepBack",
    "reverseContinue",
];

pub(crate) fn session(
    cpl: &Path,
    file: &Path,
    input: &[u8],
    header_lines: u32,
    args: &[&str],
) -> Result<ToolRun, String> {
    let path = file.to_string_lossy().into_owned();
    let thread = || obj([("threadId", 1.into())]);
    let mut lines = Vec::new();
    let mut requests = vec![
        ("initialize", obj([("adapterID", "cp".into())])),
        (
            "launch",
            obj([
                ("program", path.as_str().into()),
                ("stopOnEntry", false.into()),
                (
                    "inputText",
                    String::from_utf8_lossy(input).into_owned().into(),
                ),
            ]),
        ),
    ];
    let mut steps = Vec::new();
    for arg in args {
        if let Some(n) = arg.strip_prefix("break:") {
            let line: u32 = n
                .parse()
                .map_err(|_| format!("`break:{n}` needs a line number"))?;
            lines.push(obj([("line", (line + header_lines).into())]));
        } else if let Some(expr) = arg.strip_prefix("eval:") {
            steps.push(("evaluate", obj([("expression", expr.into())])));
        } else if STEPS.contains(arg) {
            steps.push((*arg, thread()));
        } else {
            return Err(format!(
                "unknown dap step `{arg}`: break:N, eval:EXPR, {}",
                STEPS.join(", ")
            ));
        }
    }
    let source = obj([("path", path.as_str().into())]);
    requests.push((
        "setBreakpoints",
        obj([("source", source), ("breakpoints", Value::Array(lines))]),
    ));
    requests.push(("configurationDone", obj([])));
    requests.extend(steps);
    requests.push(("disconnect", obj([])));
    let mut child = crate::pin::command(cpl)
        .args(["dap", "--stdio"])
        .current_dir(file.parent().unwrap_or(Path::new(".")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run cpl dap: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        for (seq, (command, arguments)) in requests.into_iter().enumerate() {
            let m = cpl_json::to_string(&obj([
                ("seq", ((seq + 1) as u32).into()),
                ("type", "request".into()),
                ("command", command.into()),
                ("arguments", arguments),
            ]));
            let _ = write!(stdin, "Content-Length: {}\r\n\r\n{m}", m.len());
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("cpl dap failed: {e}"))?;
    let mut stdout = String::new();
    for body in frames(&out.stdout)? {
        let value = cpl_json::parse(&body).map_err(|e| format!("cpl dap sent bad JSON: {e}"))?;
        stdout.push_str(&cpl_json::to_string(&value).replace(&path, "<case>"));
        stdout.push('\n');
    }
    Ok(ToolRun {
        status: out.status.code().unwrap_or(-1),
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        rewritten: String::new(),
    })
}

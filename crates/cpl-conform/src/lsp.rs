//! `// tool: lsp <feature> [<needle>]` cases: open the case in `cpl lsp --stdio`
//! as one document and print what the server sends back, one JSON message
//! per line, for `// has:` / `// lacks:` to check. Features: `diagnostics`
//! (the published diagnostics only), `hover`, `completion`, `definition`
//! (each at the needle's `|`, in the first non-comment line holding it),
//! `formatting` and `inlay` (the whole document's inlay hints). The document's URI prints as `<case>`.

use crate::conformance::ToolRun;
use cpl_json::{obj, Value};
use std::io::Write;
use std::path::Path;
use std::process::Stdio;

const REQUESTS: [(&str, &str); 5] = [
    ("hover", "textDocument/hover"),
    ("completion", "textDocument/completion"),
    ("definition", "textDocument/definition"),
    ("formatting", "textDocument/formatting"),
    ("inlay", "textDocument/inlayHint"),
];

pub(crate) fn session(
    cpl: &Path,
    file: &Path,
    text: &str,
    args: &[&str],
) -> Result<ToolRun, String> {
    let feature = *args.first().ok_or(
        "`// tool: lsp` needs a feature: diagnostics, hover, completion, definition, formatting or inlay",
    )?;
    let uri = file_uri(file);
    let doc = || obj([("uri", uri.as_str().into())]);
    let mut messages = vec![
        message(Some(1), "initialize", obj([("capabilities", obj([]))])),
        message(None, "initialized", obj([])),
        message(
            None,
            "textDocument/didOpen",
            obj([(
                "textDocument",
                obj([
                    ("uri", uri.as_str().into()),
                    ("languageId", "cp".into()),
                    ("version", 1.into()),
                    ("text", text.into()),
                ]),
            )]),
        ),
    ];
    if feature != "diagnostics" {
        let method = REQUESTS
            .iter()
            .find(|(f, _)| *f == feature)
            .map(|(_, m)| *m)
            .ok_or_else(|| format!("unknown lsp feature `{feature}`"))?;
        let params = if feature == "formatting" {
            let options = obj([("tabSize", 4.into()), ("insertSpaces", true.into())]);
            obj([("textDocument", doc()), ("options", options)])
        } else if feature == "inlay" {
            obj([("textDocument", doc())])
        } else {
            let needle = args.get(1).ok_or_else(|| {
                format!("`lsp {feature}` needs a needle with a `|` cursor, like `h.|`")
            })?;
            let (line, character) = cursor(text, needle)?;
            let position = obj([("line", line.into()), ("character", character.into())]);
            obj([("textDocument", doc()), ("position", position)])
        };
        messages.push(message(Some(2), method, params));
    }
    messages.push(message(Some(3), "shutdown", Value::Null));
    messages.push(message(None, "exit", Value::Null));
    let mut child = crate::pin::command(cpl)
        .args(["lsp", "--stdio"])
        .current_dir(file.parent().unwrap_or(Path::new(".")))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("cannot run cpl lsp: {e}"))?;
    if let Some(mut stdin) = child.stdin.take() {
        for m in &messages {
            let _ = write!(stdin, "Content-Length: {}\r\n\r\n{m}", m.len());
        }
    }
    let out = child
        .wait_with_output()
        .map_err(|e| format!("cpl lsp failed: {e}"))?;
    let mut stdout = String::new();
    for body in frames(&out.stdout)? {
        let value = cpl_json::parse(&body).map_err(|e| format!("cpl lsp sent bad JSON: {e}"))?;
        let wanted = value.at(&["id"]).as_u64() == Some(2)
            || value.at(&["method"]).as_str() == Some("textDocument/publishDiagnostics");
        if wanted {
            stdout.push_str(&cpl_json::to_string(&value).replace(&uri, "<case>"));
            stdout.push('\n');
        }
    }
    Ok(ToolRun {
        status: out.status.code().unwrap_or(-1),
        stdout,
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
        rewritten: String::new(),
    })
}

fn message(id: Option<u32>, method: &str, params: Value) -> String {
    let mut m = std::collections::BTreeMap::new();
    m.insert("jsonrpc".to_string(), Value::from("2.0"));
    if let Some(id) = id {
        m.insert("id".to_string(), id.into());
    }
    m.insert("method".to_string(), method.into());
    m.insert("params".to_string(), params);
    cpl_json::to_string(&Value::Object(m))
}

fn file_uri(file: &Path) -> String {
    let path = file.to_string_lossy().replace('\\', "/");
    if path.starts_with('/') {
        format!("file://{path}")
    } else {
        format!("file:///{path}")
    }
}

/// The zero-based line and column of the `|` in `needle`, found in the first
/// line that is not a `//` header or comment.
fn cursor(text: &str, needle: &str) -> Result<(u32, u32), String> {
    let bar = needle.find('|').unwrap_or(0);
    let plain = needle.replacen('|', "", 1);
    for (line, l) in text.split('\n').enumerate() {
        if l.trim_start().starts_with("//") {
            continue;
        }
        if let Some(col) = l.find(&plain) {
            return Ok((line as u32, (col + bar) as u32));
        }
    }
    Err(format!("lsp needle `{plain}` is not in the case"))
}

/// Splits `Content-Length` framed messages.
pub(crate) fn frames(bytes: &[u8]) -> Result<Vec<String>, String> {
    let mut rest = bytes;
    let mut out = Vec::new();
    while let Some(end) = rest.windows(4).position(|w| w == b"\r\n\r\n") {
        let head = String::from_utf8_lossy(&rest[..end]);
        let len: usize = head
            .lines()
            .find_map(|l| l.strip_prefix("Content-Length:"))
            .and_then(|n| n.trim().parse().ok())
            .ok_or_else(|| format!("the server sent a frame without Content-Length: {head}"))?;
        let body = rest
            .get(end + 4..end + 4 + len)
            .ok_or("the server sent a short frame")?;
        out.push(String::from_utf8_lossy(body).into_owned());
        rest = &rest[end + 4 + len..];
    }
    Ok(out)
}

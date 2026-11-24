//! `cpl.import`, Import a Problem: the server waits for Competitive
//! Companion, creates each problem it sends and asks the editor to open it
//! (`window/showDocument`), so the editor only sends the command.

use super::{Server, INVALID_PARAMS};
use crate::uri::uri_of_path;
use cpl_json::{obj, Value};
use std::path::{Path, PathBuf};

const REQUEST_FAILED: i32 = -32803;

impl Server {
    /// `workspace/executeCommand` of `cpl.import` with arguments
    /// `[path, port?]`: problems land in `path`, or the folder of the file
    /// it names. Answers `{port}` once it listens.
    pub(super) fn import(&self, arguments: &[Value]) -> Result<Value, (i32, String)> {
        let outbox = self.outbox.clone().ok_or((
            REQUEST_FAILED,
            "this server has no way to open the files it would create".to_string(),
        ))?;
        let path = arguments.first().and_then(Value::as_str).ok_or((
            INVALID_PARAMS,
            "`cpl.import` takes the folder the problems go in".to_string(),
        ))?;
        let path = PathBuf::from(path);
        let dir = if path.is_file() {
            path.parent().map(Path::to_path_buf).unwrap_or_default()
        } else {
            path
        };
        let port = match arguments.get(1) {
            None | Some(Value::Null) => cpl_contest::new::COMPANION_PORT,
            Some(port) => port
                .as_u64()
                .and_then(|p| u16::try_from(p).ok())
                .ok_or((INVALID_PARAMS, format!("`{port}` is not a port")))?,
        };
        let listener = std::net::TcpListener::bind(("127.0.0.1", port)).map_err(|e| {
            (
                REQUEST_FAILED,
                format!("can't listen on 127.0.0.1:{port}: {e}"),
            )
        })?;
        let port = listener.local_addr().map_or(port, |a| a.port());
        let say = move |kind: i64, message: String| {
            let params = obj([("type", kind.into()), ("message", message.into())]);
            notification("window/showMessage", params)
        };
        let _ = outbox.send(say(
            3,
            format!("Waiting for Competitive Companion on port {port}: open the problem in the browser and click the extension."),
        ));
        std::thread::spawn(move || {
            let mut opened = 0;
            let heard =
                cpl_contest::new::import(listener, &dir, &mut std::io::sink(), &mut |source| {
                    opened += 1;
                    let uri = uri_of_path(&source.to_string_lossy());
                    let params = obj([("uri", uri.into()), ("takeFocus", true.into())]);
                    let mut request = notification("window/showDocument", params);
                    if let Some(fields) = request.as_object_mut() {
                        fields.insert("id".into(), format!("cpl.import/{opened}").into());
                    }
                    let _ = outbox.send(request);
                });
            if let Err(message) = heard {
                let _ = outbox.send(say(1, format!("No problem imported: {message}")));
            }
        });
        Ok(obj([("port", i64::from(port).into())]))
    }
}

fn notification(method: &str, params: Value) -> Value {
    obj([
        ("jsonrpc", "2.0".into()),
        ("method", method.into()),
        ("params", params),
    ])
}

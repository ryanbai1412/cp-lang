//! Navigation requests past go-to-definition: type definition,
//! implementations, links on `use` lines, and pulled diagnostics.

use super::{range_json, Reply, Server, INVALID_PARAMS};
use crate::uri::uri_of_path;
use cpl_check::Definition;
use cpl_diag::{LineIndex, Span};
use cpl_json::{obj, Value};

impl Server {
    /// The answer to a navigation request, or `None` for another method.
    pub(super) fn navigate(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Option<Result<Reply, (i32, String)>> {
        Some(match method {
            "textDocument/typeDefinition" => self.position(params).map(|(uri, offset)| {
                let def = self.db.type_definition(&uri, offset);
                Reply::result(def.map_or(Value::Null, |def| self.location(&uri, &def)))
            }),
            "textDocument/implementation" => self.position(params).map(|(uri, offset)| {
                let defs = self.db.implementations(&uri, offset);
                Reply::result(Value::Array(
                    defs.iter().map(|def| self.location(&uri, def)).collect(),
                ))
            }),
            "textDocument/linkedEditingRange" => self.position(params).map(|(uri, offset)| {
                // Only a name declared in this file is safe to edit as one.
                let Some((target, spans)) = crate::rename::references(&mut self.db, &uri, offset)
                else {
                    return Reply::default();
                };
                let analysis = self.db.analyze(&uri).expect("references imply an analysis");
                if !crate::rename::same_file(&target.path, &uri) || spans.len() < 2 {
                    return Reply::default();
                }
                let ranges = spans
                    .iter()
                    .map(|&span| range_json(span, &analysis.text, &analysis.lines))
                    .collect::<Vec<_>>();
                Reply::result(obj([("ranges", ranges.into())]))
            }),
            "textDocument/documentLink" => super::uri_of(params).map(|uri| {
                let uri = uri.to_string();
                Reply::result(Value::Array(self.use_links(&uri)))
            }),
            // Answered for clients that ask; not announced, since a client
            // given both would show every diagnostic twice.
            "textDocument/diagnostic" => super::uri_of(params).and_then(|uri| {
                if self.db.analyze(uri).is_none() {
                    return Err((INVALID_PARAMS, format!("`{uri}` is not open")));
                }
                let published = self.diagnostics(uri);
                Ok(Reply::result(obj([
                    ("kind", "full".into()),
                    ("items", published.at(&["params", "diagnostics"]).clone()),
                ])))
            }),
            _ => return None,
        })
    }

    /// `def` as an LSP `Location`, from the open document `uri`.
    pub(super) fn location(&self, uri: &str, def: &Definition) -> Value {
        // Only the target's text: analyzing a module the file merely
        // imports would check all of it again, as if it were open.
        let text = self
            .db
            .file_text(&def.path)
            .expect("definition implies a loaded file");
        let target = if def.path == uri {
            def.path.clone()
        } else {
            uri_of_path(&def.path)
        };
        obj([
            ("uri", target.into()),
            ("range", range_json(def.span, &text, &LineIndex::new(&text))),
        ])
    }

    /// A link from the module path of each `use` line to that module's file.
    fn use_links(&mut self, uri: &str) -> Vec<Value> {
        let Some(analysis) = self.db.analyze(uri) else {
            return Vec::new();
        };
        let mut links = Vec::new();
        let mut start = 0;
        for line in analysis.text.split_inclusive('\n') {
            let at = start;
            start += line.len();
            let body = line.trim_start();
            let Some(rest) = body.strip_prefix("use ") else {
                continue;
            };
            let rest = rest.trim_start();
            let len = rest
                .find(|c: char| !(c.is_alphanumeric() || c == '_' || c == '.'))
                .unwrap_or(rest.len());
            let name = rest[..len].trim_end_matches('.');
            let Some(module) = analysis.modules().iter().find(|m| m.name == name) else {
                continue;
            };
            let from = (at + line.len() - rest.len()) as u32;
            let span = Span::new(from, from + name.len() as u32);
            links.push(obj([
                ("range", range_json(span, &analysis.text, &analysis.lines)),
                ("target", uri_of_path(&module.path).into()),
                ("tooltip", format!("Open {name}").into()),
            ]));
        }
        links
    }
}

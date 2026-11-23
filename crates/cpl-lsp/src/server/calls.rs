//! Call hierarchy: who calls a fn, and what it calls.

use super::{range_json, Reply, Server, INVALID_PARAMS};
use crate::rename;
use crate::uri::uri_of_path;
use cpl_check::Definition;
use cpl_diag::{LineIndex, Span};
use cpl_json::{obj, Value};
use std::collections::BTreeMap;

impl Server {
    /// The answer to a call hierarchy request, or `None` for another method.
    pub(super) fn calls(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Option<Result<Reply, (i32, String)>> {
        Some(match method {
            "textDocument/prepareCallHierarchy" => self.position(params).map(|(uri, offset)| {
                let def = self.db.definition_location(&uri, offset);
                let item = def.and_then(|def| self.fn_item(&uri, &def));
                Reply::result(item.map_or(Value::Null, |item| Value::Array(vec![item])))
            }),
            "callHierarchy/incomingCalls" => {
                item_of(params).map(|(target, name)| Reply::result(self.incoming(&target, &name)))
            }
            "callHierarchy/outgoingCalls" => {
                item_of(params).map(|(target, _)| Reply::result(self.outgoing(&target)))
            }
            _ => return None,
        })
    }

    /// The `CallHierarchyItem` of the fn declared at `def`, if it is one.
    fn fn_item(&mut self, uri: &str, def: &Definition) -> Option<Value> {
        let symbol = self
            .db
            .fns(&def.path)
            .into_iter()
            .find(|s| s.name_span == def.span)?;
        Some(self.item(
            uri,
            &def.path,
            &symbol.name,
            12,
            symbol.span,
            symbol.name_span,
        ))
    }

    fn item(&self, uri: &str, path: &str, name: &str, kind: u32, span: Span, at: Span) -> Value {
        let text = self.db.file_text(path).expect("a loaded file");
        let lines = LineIndex::new(&text);
        let target = if path == uri {
            path.to_string()
        } else {
            uri_of_path(path)
        };
        obj([
            ("name", name.into()),
            ("kind", kind.into()),
            ("uri", target.into()),
            ("range", range_json(span, &text, &lines)),
            ("selectionRange", range_json(at, &text, &lines)),
            (
                "data",
                obj([
                    ("path", path.into()),
                    ("start", at.start.into()),
                    ("end", at.end.into()),
                ]),
            ),
        ])
    }

    /// Each caller of `target` in the open documents, with where it calls:
    /// a fn, or the top-level code of a file.
    fn incoming(&mut self, target: &Definition, name: &str) -> Value {
        let mut out = Vec::new();
        let docs: Vec<String> = self.docs.keys().cloned().collect();
        for doc in docs {
            let uses: Vec<Span> = rename::uses_in(&mut self.db, &doc, target, name)
                .into_iter()
                .filter(|&span| !(doc == target.path && span == target.span))
                .collect();
            let fns = self.db.fns(&doc);
            let mut callers: BTreeMap<Option<usize>, Vec<Span>> = BTreeMap::new();
            for span in uses {
                let caller = fns
                    .iter()
                    .position(|f| f.span.start <= span.start && span.end <= f.span.end);
                callers.entry(caller).or_default().push(span);
            }
            let Some(text) = self.db.file_text(&doc) else {
                continue;
            };
            let lines = LineIndex::new(&text);
            // In the order they first call, as a reader meets them.
            let mut callers: Vec<_> = callers.into_iter().collect();
            callers.sort_by_key(|(_, spans)| spans[0].start);
            for (caller, spans) in callers {
                let from = match caller {
                    Some(i) => {
                        let f = &fns[i];
                        self.item(&doc, &doc, &f.name, 12, f.span, f.name_span)
                    }
                    None => {
                        let whole = Span::new(spans[0].start, spans[spans.len() - 1].end);
                        self.item(&doc, &doc, "top level", 2, whole, spans[0])
                    }
                };
                let ranges = spans
                    .iter()
                    .map(|&s| range_json(s, &text, &lines))
                    .collect();
                out.push(obj([("from", from), ("fromRanges", Value::Array(ranges))]));
            }
        }
        Value::Array(out)
    }

    /// Each fn the fn at `target` calls, with where it calls it.
    fn outgoing(&mut self, target: &Definition) -> Value {
        let path = target.path.clone();
        let Some(body) = self
            .db
            .fns(&path)
            .into_iter()
            .find(|s| s.name_span == target.span)
            .map(|s| s.span)
        else {
            return Value::Array(Vec::new());
        };
        let Some(analysis) = self.db.analyze(&path) else {
            return Value::Array(Vec::new());
        };
        // Every name in the body that resolves to a fn: plain calls, method
        // calls and fns passed as values alike.
        let word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
        let bytes = analysis.text.as_bytes();
        let calls: Vec<u32> = (body.start..body.end)
            .filter(|&at| {
                let i = at as usize;
                let b = bytes[i];
                (b.is_ascii_alphabetic() || b == b'_') && (i == 0 || !word(bytes[i - 1]))
            })
            .filter(|&at| at != target.span.start)
            .collect();
        let mut found: BTreeMap<(String, u32), (Value, Vec<Span>)> = BTreeMap::new();
        for at in calls {
            let Some(def) = self.db.definition_location(&path, at) else {
                continue;
            };
            if def == *target {
                continue;
            }
            let key = (def.path.clone(), def.span.start);
            if !found.contains_key(&key) {
                let Some(item) = self.fn_item(&path, &def) else {
                    continue;
                };
                found.insert(key.clone(), (item, Vec::new()));
            }
            let span = rename::word_at(&analysis.text, at).map_or(Span::new(at, at), |(s, _)| s);
            found.get_mut(&key).expect("inserted").1.push(span);
        }
        Value::Array(
            found
                .into_values()
                .map(|(to, spans)| {
                    let ranges = spans
                        .iter()
                        .map(|&s| range_json(s, &analysis.text, &analysis.lines))
                        .collect();
                    obj([("to", to), ("fromRanges", Value::Array(ranges))])
                })
                .collect(),
        )
    }
}

/// The declaration a `CallHierarchyItem` stands for, and its name.
fn item_of(params: &Value) -> Result<(Definition, String), (i32, String)> {
    let data = params.at(&["item", "data"]);
    let (Some(path), Some(start), Some(end)) = (
        data.at(&["path"]).as_str(),
        data.at(&["start"]).as_u32(),
        data.at(&["end"]).as_u32(),
    ) else {
        return Err((
            INVALID_PARAMS,
            "a call hierarchy item from this server".into(),
        ));
    };
    let name = params
        .at(&["item", "name"])
        .as_str()
        .unwrap_or("")
        .to_string();
    let def = Definition {
        path: path.to_string(),
        span: Span::new(start, end),
    };
    Ok((def, name))
}

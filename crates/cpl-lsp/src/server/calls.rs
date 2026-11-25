//! Call hierarchy: who calls a fn, and what it calls.

use super::{range_json, Reply, Server, INVALID_PARAMS};
use crate::rename;
use crate::uri::uri_of_path;
use cpl_diag::{LineIndex, Span};
use cpl_facade::today::Definition;
use cpl_json::{obj, Value};
use std::collections::btree_map::Entry;
use std::collections::BTreeMap;

impl Server {
    /// The answer to a call hierarchy request, or `None` for another method.
    pub(super) fn calls(
        &mut self,
        method: &str,
        params: &Value,
    ) -> Option<Result<Reply, (i32, String)>> {
        Some(match method {
            "textDocument/prepareCallHierarchy" => self.position(params).map(|(uri, offset, _)| {
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
        let text = self.db.file_text(&def.path)?;
        Some(item(
            uri,
            &def.path,
            &text,
            &symbol.name,
            12,
            symbol.span,
            symbol.name_span,
        ))
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
            cpl_diag::sort_by_key(&mut callers, |(_, spans)| spans[0].start.into());
            for (caller, spans) in callers {
                let from = match caller {
                    Some(i) => {
                        let f = &fns[i];
                        item(&doc, &doc, &text, &f.name, 12, f.span, f.name_span)
                    }
                    None => {
                        let whole = Span::new(spans[0].start, spans[spans.len() - 1].end);
                        item(&doc, &doc, &text, "top level", 2, whole, spans[0])
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
            let spans = match found.entry(key) {
                Entry::Occupied(entry) => entry.into_mut(),
                Entry::Vacant(entry) => {
                    let Some(item) = self.fn_item(&path, &def) else {
                        continue;
                    };
                    entry.insert((item, Vec::new()))
                }
            };
            let span = rename::word_at(&analysis.text, at).map_or(Span::new(at, at), |(s, _)| s);
            spans.1.push(span);
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

/// The `CallHierarchyItem` of `name`, declared at `at` in `path`, whose text is `text`.
fn item(uri: &str, path: &str, text: &str, name: &str, kind: u32, span: Span, at: Span) -> Value {
    let lines = LineIndex::new(text);
    let target = if path == uri {
        path.to_string()
    } else {
        uri_of_path(path)
    };
    obj([
        ("name", name.into()),
        ("kind", kind.into()),
        ("uri", target.into()),
        ("range", range_json(span, text, &lines)),
        ("selectionRange", range_json(at, text, &lines)),
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

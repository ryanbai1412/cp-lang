//! Code actions: each diagnostic's fixes, `cpl fix` on the whole file,
//! and organize imports.

use super::*;

impl Server {
    pub(super) fn code_actions(&mut self, params: &Value) -> Result<Reply, (i32, String)> {
        let uri = uri_of(params)?;
        let Some(analysis) = self.db.analyze(uri) else {
            return Ok(Reply::result(Value::Array(Vec::new())));
        };
        let start = position_of(params.at(&["range", "start"]))?;
        let end = position_of(params.at(&["range", "end"]))?;
        let wanted = Span::new(
            analysis.lines.offset_utf16(start, &analysis.text),
            analysis.lines.offset_utf16(end, &analysis.text),
        );
        let mut actions = Vec::new();
        let lints = lints(uri, &analysis);
        for diag in analysis.diagnostics.iter().chain(&lints) {
            if diag.span.end < wanted.start || diag.span.start > wanted.end {
                continue;
            }
            for fix in &diag.fixes {
                let edits: Vec<Value> = fix
                    .edits
                    .iter()
                    .map(|(span, text)| {
                        let (span, text) = cpl_diag::edit_like(&analysis.text, *span, text);
                        obj([
                            ("range", range_json(span, &analysis.text, &analysis.lines)),
                            ("newText", text.into()),
                        ])
                    })
                    .collect();
                actions.push(obj([
                    ("title", fix.title.as_str().into()),
                    ("kind", "quickfix".into()),
                    (
                        "diagnostics",
                        vec![diagnostic_json(diag, uri, &analysis.text, &analysis.lines)].into(),
                    ),
                    (
                        "edit",
                        obj([(
                            "changes",
                            Value::Object(BTreeMap::from([(uri.to_string(), edits.into())])),
                        )]),
                    ),
                ]));
            }
        }
        // `cpl fix` on the document, for an editor's fix-all (on save).
        let only = params.at(&["context", "only"]).as_array().unwrap_or(&[]);
        let fix_all = only.is_empty()
            || only
                .iter()
                .filter_map(Value::as_str)
                .any(|kind| "source.fixAll".starts_with(kind) || kind.starts_with("source.fixAll"));
        let fixed = fix_all.then(|| fix_file(uri, &analysis.text)).flatten();
        if let Some(text) = fixed {
            let whole = Span::new(0, analysis.text.len() as u32);
            actions.push(obj([
                ("title", "apply every `cpl fix` fix".into()),
                ("kind", "source.fixAll.cpl".into()),
                (
                    "edit",
                    obj([(
                        "changes",
                        Value::Object(BTreeMap::from([(
                            uri.to_string(),
                            vec![obj([
                                ("range", range_json(whole, &analysis.text, &analysis.lines)),
                                ("newText", text.into()),
                            ])]
                            .into(),
                        )])),
                    )]),
                ),
            ]));
        }
        let organize = only.is_empty()
            || only.iter().filter_map(Value::as_str).any(|kind| {
                "source.organizeImports".starts_with(kind)
                    || kind.starts_with("source.organizeImports")
            });
        if let Some((span, text)) = organize
            .then(|| crate::imports::organize(&analysis.text))
            .flatten()
        {
            actions.push(obj([
                ("title", "organize imports".into()),
                ("kind", "source.organizeImports".into()),
                (
                    "edit",
                    obj([(
                        "changes",
                        Value::Object(BTreeMap::from([(
                            uri.to_string(),
                            vec![obj([
                                ("range", range_json(span, &analysis.text, &analysis.lines)),
                                ("newText", text.into()),
                            ])]
                            .into(),
                        )])),
                    )]),
                ),
            ]));
        }
        Ok(Reply::result(Value::Array(actions)))
    }
}

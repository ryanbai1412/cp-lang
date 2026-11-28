//! `cpl explain`'s storage section and hover read one computation: on 20
//! hill-climb ports, hover on each top-level value that isn't a primitive
//! states the bytes explain
//! lists for it (spec/tools/explain.md, "LSP hover shows the same data").

use cpl_facade::today::{human_bytes, DeclKind};
use cpl_facade::Session;
use std::path::Path;

const PRIMITIVES: &[&str] = &[
    "Int", "F64", "Bool", "Char", "I8", "I16", "I32", "I64", "I128", "U8", "U16", "U32", "U64",
    "U128",
];

#[test]
fn hover_states_the_storage_explain_lists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let list = std::fs::read_to_string(root.join("corpus/hillclimb.json")).unwrap();
    let list = cpl_json::parse(&list).unwrap();
    let ids: Vec<&str> = list
        .at(&["problems"])
        .as_array()
        .unwrap_or(&[])
        .iter()
        .filter_map(|p| p.as_str())
        .take(20)
        .collect();
    assert_eq!(ids.len(), 20);

    let (mut compared, mut failures) = (0, Vec::new());
    for pid in ids {
        let path = format!("{pid}.cp");
        let src =
            std::fs::read_to_string(root.join(format!("corpus/problems/{pid}/spec.cp"))).unwrap();
        let mut db = Session::default();
        db = db.with_file(&path, src.as_str());
        let explained = cpl_lsp::explain::storage(&mut db, &path).rows;
        for symbol in db.symbols(&path) {
            if !matches!(symbol.kind, DeclKind::Value) {
                continue;
            }
            let Some((_, rep)) = explained.iter().find(|(name, _)| *name == symbol.name) else {
                continue;
            };
            // A primitive's hover is its type, which already names its width.
            let primitive = PRIMITIVES.contains(&rep.ty.as_str());
            let Some(bytes) = rep.bytes.filter(|_| !primitive) else {
                continue;
            };
            let want = match rep.max_bytes {
                Some(most) if rep.max_cells.is_some() => format!(
                    "storage: {} per element, {} max",
                    human_bytes(bytes.into()),
                    human_bytes(most)
                ),
                _ if rep.max_cells.is_some() || rep.ty.starts_with('[') => {
                    format!("storage: {} per element", human_bytes(bytes.into()))
                }
                _ => format!("storage: {}", human_bytes(bytes.into())),
            };
            let hover = db
                .hover(&path, symbol.name_span.start)
                .map(|h| h.text)
                .unwrap_or_default();
            compared += 1;
            if !hover.contains(&want) {
                failures.push(format!(
                    "{pid} `{}`: explain says {want:?}, hover says {hover:?}",
                    symbol.name
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(compared >= 20, "only {compared} values compared");
}

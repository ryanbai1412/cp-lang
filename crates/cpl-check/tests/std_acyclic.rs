use std::path::PathBuf;

use cpl_check::Database;

#[test]
fn no_spec_std_module_imports_itself_back() {
    let std_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let std_dir = std_dir.canonicalize().expect("std exists");
    // SAFETY: the only test in this binary, set before anything reads them.
    unsafe {
        std::env::set_var("CPL_SPEC", "1");
        std::env::set_var("CPL_STD", &std_dir);
    }
    let mut files: Vec<PathBuf> = std::fs::read_dir(&std_dir)
        .unwrap()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "cp"))
        .collect();
    files.sort();
    let mut cycles: Vec<String> = Vec::new();
    for file in &files {
        let module = file.file_stem().unwrap().to_string_lossy();
        if module == "prelude" {
            continue;
        }
        let mut db = Database::new();
        let main = std_dir.join("../main.cp").to_string_lossy().into_owned();
        db.set_file(&main, format!("use std.{module}\n"));
        db.analyze(&main);
        for cycle in db.import_cycles() {
            let names: Vec<String> = cycle
                .iter()
                .map(|p| {
                    let stem = std::path::Path::new(p).file_stem().unwrap();
                    format!("std.{}", stem.to_string_lossy())
                })
                .collect();
            let ring = &names[..names.len() - 1];
            let start = (0..ring.len()).min_by_key(|&i| &ring[i]).unwrap();
            let mut around: Vec<&str> = ring[start..]
                .iter()
                .chain(&ring[..start])
                .map(String::as_str)
                .collect();
            around.push(around[0]);
            let line = around.join(" -> ");
            if !cycles.contains(&line) {
                cycles.push(line);
            }
        }
    }
    assert!(
        cycles.is_empty(),
        "std modules import each other:\n{}",
        cycles.join("\n")
    );
}

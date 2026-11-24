//! Helpers the integration tests and examples share.

/// The program a source file lowers to; it must check without diagnostics.
pub fn program(source: &str) -> cpl_ir::Program {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    cpl_ir::lower(&analysis).expect("lowers")
}

use std::path::Path;

use cpl_check::Database;
use cpl_conform::runner;

/// A worker reuses one `Database` across cases, so a case must compile the
/// same after any other case as it does alone.
#[test]
fn a_case_compiles_the_same_after_any_other_case() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let read = |rel: &str| {
        let path = root.join(rel);
        let text = std::fs::read_to_string(&path).unwrap();
        (path.to_string_lossy().into_owned(), text)
    };
    let (first, first_text) = read("spec/conformance/core-01/operation-values.cp");
    for victim in [
        "spec/conformance/core-07/view-walks.cp",
        "spec/conformance/core-07/with-prev.cp",
    ] {
        let (path, text) = read(victim);
        let mut db = Database::new();
        let _ = runner::compile_program(&mut db, &first, &first_text);
        if let Err(runner::CompileError::Diagnostics(err)) =
            runner::compile_program(&mut db, &path, &text)
        {
            panic!("{victim} stops compiling after operation-values.cp:\n{err}");
        }
    }
}

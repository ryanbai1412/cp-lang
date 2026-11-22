use cpl_check::Database;
use cpl_conform::conformance::{run_one, ConformanceCase};
use cpl_conform::find_repo_root;

/// One worker's `Database` runs cases in turn, so whichever case first
/// reaches a std module must not decide what that module holds for the next.
#[test]
fn a_case_does_not_change_what_std_gives_the_next_one() {
    let root = find_repo_root().unwrap();
    let mut db = Database::new();
    for id in ["std-rand/chance-pick", "std-strings/search-empty-pattern"] {
        let case = ConformanceCase {
            id: id.to_string(),
            path: root.join(format!("spec/conformance/{id}.cp")),
        };
        let outcome = run_one(&mut db, &case);
        assert_eq!(outcome.status, "pass", "{id}: {}", outcome.detail);
    }
}

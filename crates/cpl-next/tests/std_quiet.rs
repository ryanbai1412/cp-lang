//! The std check errors a program never says: an impl outside the program is checked quietly
//! when reached through its trait (check/lib.rs `reach`), its errors deferred until an instance
//! runs it. `std-quiet.tsv` holds them by module and stays equal to the run, written to
//! `target/cpl-next/std-quiet.tsv`; debt-ratchet's `next_std_quiet_errors` sums them, and the
//! quiet branch goes at 0.

use cpl_next::facade::{Session, Snapshot, World};
use std::path::Path;

#[test]
fn std_quiet_is_what_a_program_hides() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let s = Snapshot::new(World::Closed).with_file("quiet.cp", "print(1)\n");
    let found: String = (Session::open(s).quiet_errors("quiet.cp").into_iter())
        .map(|(m, n)| format!("{m}\t{n}\n"))
        .collect();
    let out = here.join("../../target/cpl-next");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("std-quiet.tsv"), &found).unwrap();
    let listed = std::fs::read_to_string(here.join("tests/std-quiet.tsv")).unwrap_or_default();
    assert!(
        listed == found,
        "tests/std-quiet.tsv is not what a program hides: copy target/cpl-next/std-quiet.tsv over it"
    );
}

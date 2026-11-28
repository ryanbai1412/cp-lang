//! Std as a program (suite 4 of plan/end-state.toml): each std module and how many
//! diagnostics the new pipeline gives once every one of its fns is checked, not only those a
//! program reaches. `std-unchecked.tsv` holds the counts and stays equal to the run, which is
//! written to `target/cpl-next/std-unchecked.tsv`; debt-ratchet's `next_std_unchecked` counts
//! the modules above 0.

use cpl_next::facade::{Session, Snapshot};
use std::path::Path;

#[test]
fn std_unchecked_is_what_checking_every_std_fn_finds() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let std = Snapshot::new(cpl_next::facade::World::Closed);
    let found: String = (Session::open(std).std_unchecked().into_iter())
        .map(|(m, ds)| format!("{m}\t{}\n", ds.len()))
        .collect();
    let out = here.join("../../target/cpl-next");
    std::fs::create_dir_all(&out).unwrap();
    std::fs::write(out.join("std-unchecked.tsv"), &found).unwrap();
    let listed = std::fs::read_to_string(here.join("tests/std-unchecked.tsv")).unwrap_or_default();
    assert!(
        listed == found,
        "tests/std-unchecked.tsv is not what checking every std fn finds: copy target/cpl-next/std-unchecked.tsv over it"
    );
}

//! Std as a program (suite 4 of plan/end-state.toml): each std module and how many
//! diagnostics the new pipeline gives once every one of its fns is checked, not only those a
//! program reaches. `std-unchecked.tsv` holds the counts and stays equal to the run, which is
//! written to `target/cpl-next/std-unchecked.tsv`; debt-ratchet's `next_std_unchecked` counts
//! the modules above 0.

use std::path::Path;

#[test]
fn std_unchecked_is_what_checking_every_std_fn_finds() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let found: String = (cpl_next::std_unchecked(&cpl_next::Std::read()).into_iter())
        .map(|(m, n)| format!("{m}\t{n}\n"))
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

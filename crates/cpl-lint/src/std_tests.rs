//! Lints that only fit a program stay quiet on the stdlib's own modules.

use super::*;

fn codes(src: &str) -> Vec<&'static str> {
    lint(src, &Config::default())
        .iter()
        .map(|d| d.code.0)
        .collect()
}

#[test]
fn the_prelude_file_is_not_its_own_redundant_import() {
    let prelude = include_str!("../../../spec/std/prelude.cp");
    assert!(!codes(prelude).contains(&REDUNDANT_PRELUDE_IMPORT.code.0));
}

#[test]
fn a_library_module_may_write_range_tree_loops() {
    for module in [
        include_str!("../../../spec/std/fenwick.cp"),
        include_str!("../../../spec/std/segtree.cp"),
    ] {
        assert!(!codes(module).contains(&HAND_ROLLED_RANGE_TREE.code.0));
    }
    let program = include_str!("../../../spec/lints/hand_rolled_range_tree/before.cp");
    assert!(codes(program).contains(&HAND_ROLLED_RANGE_TREE.code.0));
}

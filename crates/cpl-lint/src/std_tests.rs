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
    let prelude = crate::std_source!("prelude.cp");
    assert!(!codes(prelude).contains(&REDUNDANT_PRELUDE_IMPORT.code.0));
}

#[test]
fn a_library_module_may_write_range_tree_loops() {
    for module in [
        crate::std_source!("fenwick.cp"),
        crate::std_source!("segtree.cp"),
    ] {
        assert!(!codes(module).contains(&HAND_ROLLED_RANGE_TREE.code.0));
    }
    let program = include_str!("../../../spec/lints/hand_rolled_range_tree/before.cp");
    assert!(codes(program).contains(&HAND_ROLLED_RANGE_TREE.code.0));
}

#[test]
fn a_library_module_may_build_a_sieve_or_prefix_sums() {
    let math = crate::std_source!("math.cp");
    assert!(!codes(math).contains(&HAND_ROLLED_SIEVE.code.0));
    let prefix = crate::std_source!("prefix.cp");
    assert!(!codes(prefix).contains(&HAND_ROLLED_PREFIX_SUM.code.0));
}

#[test]
fn a_fn_is_not_told_to_call_itself() {
    let src = "pub fn find(xs: List<Int>) -> ?Int = xs.first(_ => true)\n";
    assert!(!codes(src).contains(&FREE_FOLD.code.0));
    let call = "let xs = [1, 2]\nprint(xs.first(_ => true))\n";
    assert!(codes(call).contains(&FREE_FOLD.code.0));
}

/// `cpl lint` refuses a module `cpl_syntax` does not parse.
#[test]
fn every_std_module_parses() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(crate::STD_DIR);
    let failing: Vec<String> = std::fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|e| e == "cp"))
        .filter(|path| {
            let src = std::fs::read_to_string(path).unwrap();
            !cpl_syntax::parse_file(&src).diagnostics.is_empty()
        })
        .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert!(
        failing.is_empty(),
        "these std modules do not parse: {failing:?}"
    );
}

#[test]
fn free_first_with_a_test_becomes_find() {
    let src = "let xs = [3, 1, 4]\nprint(first(xs, x => x > 3))\n";
    assert_eq!(codes(src), vec!["L0509"]);
    let (out, _) = fix(src, &Config::default());
    assert_eq!(out, "let xs = [3, 1, 4]\nprint(xs.find(x => x > 3))\n");
    assert!(codes(&out).is_empty());
}

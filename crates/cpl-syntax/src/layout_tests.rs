//! Recovery from indentation mistakes: one error each, with its fix.

use crate::*;

#[test]
fn an_indent_where_a_value_was_wanted_is_the_indent_error() {
    for src in [
        "@!multitest\n    let n: Int = read()\nprint(n)\n",
        "fn f(c: Char) -> Int:\n    match c:\n        'a' => 1\n            _ => 2\n",
    ] {
        let diags = parse_file(src).diagnostics;
        assert_eq!(diags.len(), 1, "{src:?}: {diags:#?}");
        let d = &diags[0];
        assert_eq!(d.code.0, "E0006", "{src:?}: {d:#?}");
        assert!(!d.fixes.is_empty(), "{src:?}: {d:#?}");
    }
}

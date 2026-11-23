//! Recovery from other languages' syntax: one error each, with its fix.

use crate::*;

#[test]
fn a_reference_type_is_one_error_with_the_fix() {
    let d = parse_file("fn f(a: &List<Int>) -> Int = a.len\n").diagnostics;
    let fixed: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixed, [""], "{d:?}");
}

#[test]
fn a_postfix_optional_is_one_error_moving_the_question_mark() {
    let d = parse_file("let x: Int? = 3\nprint(x ?? 0)\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    let edits: Vec<_> = d[0].fixes[0].edits.iter().map(|e| e.1.as_str()).collect();
    assert_eq!(edits, ["?", ""]);
}

#[test]
fn a_struct_literal_is_one_error_with_parentheses() {
    let d = parse_file("struct P(x: Int)\nlet p = P { x: 1 }\nprint(p.x)\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    let edits: Vec<_> = d[0].fixes[0].edits.iter().map(|e| e.1.as_str()).collect();
    assert_eq!(edits, ["(", ")"]);
}

#[test]
fn a_colon_return_type_is_one_error_writing_the_arrow() {
    for src in [
        "fn f(x: Int): Int = x\nprint(f(1))\n",
        "fn f(): List<Int>:\n    return [1]\n",
    ] {
        let d = parse_file(src).diagnostics;
        assert_eq!(d.len(), 1, "{src}: {d:?}");
        assert_eq!(d[0].fixes[0].edits[0].1, " ->");
    }
    assert!(parse_file("fn f():\n    print(1)\n").diagnostics.is_empty());
}

#[test]
fn a_rust_print_macro_is_one_error_with_the_print_call() {
    let fix = |src: &str| {
        let d = parse_file(src).diagnostics;
        assert_eq!(d.len(), 1, "{src}: {d:?}");
        d[0].fixes.first().map(|f| f.edits[0].1.clone())
    };
    assert_eq!(
        fix("let x = 3\nprintln!(\"{} {}\", x, x + 1)\n").as_deref(),
        Some("print(x, x + 1)")
    );
    assert_eq!(
        fix("println!(\"hi {x}\")\n").as_deref(),
        Some("print(\"hi {x}\")")
    );
    assert_eq!(fix("let x = 3\nprintln!(\"x={}\", x)\n"), None);
    assert_eq!(fix("let x = 3\neprintln!(\"{}\", x)\n"), None);
}

#[test]
fn a_swift_empty_map_is_one_error_with_the_map_type() {
    let d = parse_file("var d = [Str: Int]()\nprint(d.len)\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "Map<Str, Int>()");
    assert!(parse_file("let m = [1: 2]\nprint(m)\n")
        .diagnostics
        .is_empty());
}

#[test]
fn a_rust_repeat_is_fixed_to_a_fill() {
    let d = parse_file("let n = 3\nvar a = [0; n + 1]\nprint(a)\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    let edits: Vec<_> = d[0].fixes[0].edits.iter().map(|e| e.1.as_str()).collect();
    assert_eq!(edits, ["n + 1", ".filled(0)"]);
}

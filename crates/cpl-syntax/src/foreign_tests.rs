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

#[test]
fn a_rust_vec_type_is_one_error_with_list() {
    let d = parse_file("let xs: Vec<i64> = [1]\nprint(xs)\n").diagnostics;
    let fixes: Vec<_> = d.iter().map(|d| d.fixes[0].edits[0].1.as_str()).collect();
    assert_eq!(fixes, ["List"], "{d:?}");
    assert!(
        parse_file("use std.geometry.{Vec}\nlet v: Vec = Vec(1.0, 2.0)\nprint(v.x)\n")
            .diagnostics
            .is_empty()
    );
}

#[test]
fn pythons_type_brackets_are_one_error_with_angles() {
    let d = parse_file("let xs: List[Int] = [1]\nprint(xs)\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    let edits: Vec<_> = d[0].fixes[0].edits.iter().map(|e| e.1.as_str()).collect();
    assert_eq!(edits, ["<", ">"]);
    assert!(parse_file("let xs: List<Int> = [1]\nprint(xs[0])\n")
        .diagnostics
        .is_empty());
}

#[test]
fn javas_lambda_arrow_is_one_error_with_fat_arrow() {
    let d = parse_file("let xs = [1, 2]\nprint(xs.map(x -> x * 2))\n").diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "=>");
    assert!(parse_file("fn f(x: Int) -> Int = x\nprint(f(1))\n")
        .diagnostics
        .is_empty());
}

#[test]
fn javas_new_is_one_error_that_drops_it() {
    let src = "struct P:\n    x: Int\nlet p = new P(x: 1)\nprint(p.x)\n";
    let d = parse_file(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    assert_eq!(d[0].fixes[0].edits[0].1, "");
    assert!(parse_file("let new = 1\nprint(new)\n")
        .diagnostics
        .is_empty());
}

#[test]
fn a_c_brace_block_header_becomes_a_colon_block() {
    let src = "let x = 3\nif x > 2 {\n    print(x)\n}\n";
    let d = parse_file(src).diagnostics;
    assert_eq!(d.len(), 1, "{d:?}");
    let edits = &d[0].fixes[0].edits;
    assert_eq!(edits.len(), 2, "{edits:?}");
    assert_eq!(
        (edits[0].0.start, edits[0].0.end, edits[0].1.as_str()),
        (18, 20, ":")
    );
    assert_eq!(
        (edits[1].0.start, edits[1].0.end, edits[1].1.as_str()),
        (33, 35, "")
    );
    let src = "let x = 3\nif x > 2 {\n    print(x)\n} else {\n    print(0)\n}\n";
    let edits = &parse_file(src).diagnostics[0].fixes[0].edits;
    assert_eq!((edits[1].0.start, edits[1].0.end), (34, 36), "{edits:?}");
    let one_line = parse_file("let x = 3\nif x > 2 {\n    print(x) }\n").diagnostics;
    assert_eq!(one_line[0].fixes[0].edits.len(), 1);
}

#[test]
fn brace_bodied_fn_and_struct_get_the_brace_fix() {
    let src = "struct P {\n    x: Int\n}\nfn f(p: P) -> Int {\n    return p.x\n}\n";
    let d = parse_file(src).diagnostics;
    assert_eq!(d.len(), 2, "{d:?}");
    assert!(d.iter().all(|d| d.fixes[0].edits.len() == 2), "{d:?}");
}

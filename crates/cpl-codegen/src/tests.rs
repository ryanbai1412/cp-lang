//! The shape of the C++: what the differential tests can't see, because
//! slower or uglier C++ prints the same answers.

use crate::emit;

fn cpp(source: &str) -> String {
    let mut db = cpl_check::Database::new();
    db.set_file("t.cp", source);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = cpl_ir::lower(&analysis).expect("lowers");
    emit(&program, "t.cp", source)
}

/// The statements of the program: the C++ after the runtime it leans on.
fn body(source: &str) -> String {
    let cpp = cpp(source);
    let start = cpp.find("void solve()").expect("a solve fn");
    cpp[start..].to_string()
}

#[test]
fn division_by_a_positive_number_of_a_non_negative_one_is_plain_cpp() {
    let body = body(
        "let a: 0..=100 = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b, a % 1000000007)\n",
    );
    assert!(body.contains("cpl::put_int(a / b)"), "{body}");
    assert!(body.contains("cpl::put_int(a % b)"), "{body}");
    assert!(body.contains("cpl::put_int(a % 1000000007)"), "{body}");
    assert!(
        !body.contains("cpl::ediv") && !body.contains("cpl::divp"),
        "{body}"
    );
}

#[test]
fn a_dividend_that_may_be_negative_gets_the_branchless_fix() {
    let source = "let a: Int = read()\nlet b: 1..=100 = read()\nprint(a / b, a % b)\n";
    let body = body(source);
    assert!(
        body.contains("cpl::divp(a, b)") && body.contains("cpl::modp(a, b)"),
        "{body}"
    );
    let cpp = cpp(source);
    assert!(
        cpp.contains("inline ll modp(ll a, ll b) { ll r = a % b; return r + ((r >> 63) & b); }"),
        "{cpp}"
    );
    // The helpers it does not call are not in the file.
    assert!(!cpp.contains("inline ll emod"), "{cpp}");
}

#[test]
fn a_divisor_of_unknown_sign_gets_the_full_euclidean_helpers() {
    let body = body("let a: Int = read()\nlet b: Int = read()\nprint(a / b, a % b)\n");
    assert!(
        body.contains("cpl::ediv(a, b)") && body.contains("cpl::emod(a, b)"),
        "{body}"
    );
}

#[test]
fn a_variable_assigned_later_is_not_trusted_to_keep_its_range() {
    let body = body("var a: 0..=100 = read()\na -= 500\nprint(a % 7)\n");
    assert!(body.contains("cpl::modp(a, 7)"), "{body}");
}

#[test]
fn loop_counters_and_lengths_are_known_non_negative() {
    let body =
        body("let xs: List<Int> = [1, 2, 3]\nfor i in 0..10:\n    print(i % 3, xs.len / 2)\n");
    assert!(body.contains("cpl::put_int(i % 3)"), "{body}");
    assert!(body.contains("(ll)xs.size() / 2"), "{body}");
}

#[test]
fn the_file_opens_with_its_source_as_comments() {
    let source = "// ends in a backslash \\\nprint(1)\n";
    let cpp = cpp(source);
    assert!(cpp.starts_with("// t.cp: C++17 written by cpl "), "{cpp}");
    assert!(cpp.contains("//     print(1)\n"), "{cpp}");
    // A `\` at the end of a `//` line would make the next line a comment too.
    assert!(cpp.lines().all(|line| !line.ends_with('\\')), "{cpp}");
}

#[test]
fn only_the_runtime_a_program_uses_is_written() {
    let hello = cpp("print(\"hello\")\n");
    assert!(
        !hello.contains("read_int")
            && !hello.contains("struct Grid")
            && !hello.contains("put_float"),
        "{hello}"
    );
    assert!(hello.len() < 2000, "{} bytes for hello world", hello.len());
}

#[test]
fn strings_are_bytes_with_no_encoding_machinery() {
    let cpp = cpp("let s: Str = read()\nvar t: Str = \"\"\nfor c in s:\n    t.push(c)\nprint(t, s[0], s.len)\n");
    assert!(cpp.contains("using u8 = unsigned char;"), "{cpp}");
    for word in [
        "wchar", "char32_t", "char16_t", "codecvt", "locale", "utf", "UTF",
    ] {
        assert!(
            !cpp[cpp.find("#include").expect("includes")..].contains(word),
            "`{word}` in {cpp}"
        );
    }
}

#[test]
fn bool_elements_are_bytes_not_vector_bool() {
    let cpp = cpp("var seen: [10]Bool = false\nseen[3] = true\nprint(seen[3])\n");
    assert!(
        cpp.contains("std::vector<char>") && !cpp.contains("std::vector<bool>"),
        "{cpp}"
    );
}

#[test]
fn the_same_source_gives_the_same_bytes() {
    let source = "var m: Map<Str, Int> = []\nm[\"b\"] = 1\nm[\"a\"] = 2\nvar s: Set<Int> = []\ns.insert(3)\nprint(m, s)\n";
    assert_eq!(cpp(source), cpp(source));
}

//! Lowered programs run to the output the source asks for, on the paths
//! lowering once trusted with a panic.

fn runs(source: &str, input: &str) -> String {
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let Some(analysis) = db.analyze("t.cp") else {
        return "no analysis".into();
    };
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let program = match cpl_ir::lower(&analysis) {
        Ok(program) => program,
        Err(diags) => return format!("{diags:?}"),
    };
    let mut out = Vec::new();
    if let Err(trap) = cpl_interp::run(&program, input.as_bytes(), &mut out) {
        return trap.into_message();
    }
    String::from_utf8_lossy(&out).into_owned()
}

#[test]
fn sequence_methods_take_their_element_from_the_type() {
    let source = "let xs = [3, 1, 2]\n\
        print(xs.get(1)!)\n\
        print(xs.at(0))\n\
        print(\"abc\".nth(1))\n\
        print(xs.find(x => x > 1) ?? 0)\n\
        print(xs.last(1) ?? 0)\n\
        print(sorted(0..3))\n\
        print(sorted(\"cab\"))\n";
    assert_eq!(runs(source, ""), "1\n3\nb\n3\n1\n0 1 2\nabc\n");
}

#[test]
fn a_reversed_range_walks_its_own_element() {
    let source = "let xs = [3, 1, 2]\nprint((0..3).rev)\nprint(xs.keys.rev)\n";
    assert_eq!(runs(source, ""), "2 1 0\n2 1 0\n");
}

#[test]
fn a_tagged_enum_reads_each_variant_by_its_one_tag() {
    let source = "enum Move = @tag(\"L\") Left | @tag(\"R\") Right\n\
        let ms: [2]Move = read()\n\
        for m in ms:\n    print(yes(m == Move.Left))\n";
    assert_eq!(runs(source, "R L\n"), "NO\nYES\n");
}

#[test]
fn a_min_over_optionals_starts_from_its_first_plain_value() {
    let source =
        "let a: ?Int = None\nlet b: ?Int = Some(1)\nprint(min(a, 5, 3))\nprint(min(b, 5))\n";
    assert_eq!(runs(source, ""), "3\n1\n");
}

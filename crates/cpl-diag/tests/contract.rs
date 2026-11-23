//! The parts of a diagnostic every tool shares, pinned at the edges the
//! error-quality suites only reach end to end: JSON positions and fields,
//! page links, fix application, and the rendered gutter and summary.
//! Rule: `errors/R12`, handbook/guidelines/errors-checklist.md#r12-every-tool-shows-the-same-diagnostic.

use cpl_diag::{apply_fixes, json, registry::page, render_all, Code, Diagnostic, LineIndex, Span};

fn field<'a>(v: &'a cpl_json::Value, key: &str) -> &'a cpl_json::Value {
    match v {
        cpl_json::Value::Object(fields) => fields
            .get(key)
            .unwrap_or_else(|| panic!("no `{key}` in {v}")),
        _ => panic!("{v} is not an object"),
    }
}

#[test]
fn json_gives_one_based_positions_offsets_and_the_page() {
    let src = "let x = 1\nfoo\n";
    let mut d = Diagnostic::error(Code("E0201"), Span::new(10, 13), "no `foo`");
    d.fix("rename", Span::new(10, 13), "x");
    let v = json::diagnostic(&d, &LineIndex::new(src));
    let at = |end: &str, key: &str| field(field(&v, end), key).to_string();
    assert_eq!(
        [
            at("start", "line"),
            at("start", "col"),
            at("start", "offset")
        ],
        ["2", "1", "10"]
    );
    assert_eq!(
        [at("end", "line"), at("end", "col"), at("end", "offset")],
        ["2", "4", "13"]
    );
    assert_eq!(field(&v, "code").to_string(), "\"E0201\"");
    assert_eq!(field(&v, "message").to_string(), "\"no `foo`\"");
    assert_eq!(field(&v, "severity").to_string(), "\"error\"");
    assert_eq!(
        field(&v, "page").to_string(),
        "\"https://cp-lang.org/errors/E0201\""
    );
    assert!(
        field(&v, "fixes").to_string().contains("\"text\":\"x\""),
        "{v}"
    );
}

#[test]
fn every_well_formed_code_has_its_section_and_nothing_else_does() {
    assert_eq!(
        page("L0306").as_deref(),
        Some("https://cp-lang.org/lints/L0306")
    );
    for code in ["E0201", "W0101", "R0101"] {
        assert_eq!(
            page(code),
            Some(format!("https://cp-lang.org/errors/{code}"))
        );
    }
    for code in ["X0201", "E020", "E02011", "E02a1", ""] {
        assert_eq!(page(code), None, "{code}");
    }
}

fn fixed(src: &str, fixes: &[(u32, u32, &str)]) -> (String, usize) {
    let diags: Vec<Diagnostic> = fixes
        .iter()
        .map(|&(a, b, text)| {
            let mut d = Diagnostic::error(Code("E0201"), Span::new(a.min(b), a.min(b)), "x");
            d.fix("fix", Span { start: a, end: b }, text);
            d
        })
        .collect();
    apply_fixes(src, &diags)
}

#[test]
fn fixes_apply_side_by_side_but_not_over_each_other_or_outside_the_source() {
    assert_eq!(
        fixed("abcdef", &[(0, 3, "X"), (3, 5, "Y")]),
        ("XYf".into(), 2)
    );
    assert_eq!(
        fixed("abcdef", &[(0, 3, "X"), (2, 5, "Y")]),
        ("Xdef".into(), 1)
    );
    assert_eq!(fixed("abcdef", &[(4, 2, "X")]), ("abcdef".into(), 0));
    assert_eq!(fixed("aé", &[(2, 3, "X")]), ("aé".into(), 0));
    assert_eq!(fixed("aé", &[(1, 2, "X")]), ("aé".into(), 0));
    assert_eq!(fixed("aé", &[(1, 3, "e")]), ("ae".into(), 1));
}

#[test]
fn the_gutter_fits_the_line_number_and_the_summary_counts_each_severity() {
    let src = "a\n".repeat(9) + "boom\n";
    let at = src.find("boom").unwrap() as u32;
    let err = Diagnostic::error(Code("E0201"), Span::new(at, at + 4), "bad");
    let warn = Diagnostic::warning(Code("L0306"), Span::new(0, 1), "odd");
    let out = render_all(std::slice::from_ref(&err), "f.cp", &src);
    assert!(out.lines().any(|l| l == "   |"), "{out}");
    assert!(out.contains("10 | boom"), "{out}");
    assert!(out.ends_with("f.cp: 1 error\n"), "{out}");
    let out = render_all(&[err.clone(), err.clone(), warn.clone()], "f.cp", &src);
    assert!(
        out.contains("2 errors") && out.contains("1 warning"),
        "{out}"
    );
    let out = render_all(&[warn], "f.cp", &src);
    assert!(out.contains("1 warning") && !out.contains("error"), "{out}");
}

#[test]
fn edit_distance_counts_a_neighbour_swap_as_one_edit() {
    use cpl_diag::edit_distance as d;
    let cases = [
        ("raods", "roads", 1),
        ("ab", "ba", 1),
        ("abcd", "acbd", 1),
        ("kitten", "sitting", 3),
        ("flaw", "lawn", 2),
        ("ca", "abc", 3),
        ("", "abc", 3),
        ("abc", "", 3),
        ("abc", "abc", 0),
        ("é", "e", 1),
    ];
    for (a, b, want) in cases {
        assert_eq!(d(a, b), want, "{a} -> {b}");
    }
}

#[test]
fn a_span_knows_its_length() {
    assert_eq!(
        (Span::new(2, 5).len(), Span::new(2, 5).is_empty()),
        (3, false)
    );
    assert_eq!((Span::point(4).len(), Span::point(4).is_empty()), (0, true));
}

#[test]
fn a_suggestion_is_help_and_a_fix_and_fixes_keep_the_files_line_endings() {
    let mut d = Diagnostic::error(Code("E0201"), Span::new(0, 3), "no `fo`");
    d.suggest(Span::new(0, 3), "foo");
    assert_eq!(d.help.as_deref(), Some("did you mean `foo`?"));
    assert_eq!(d.fixes[0].title, "change to `foo`");
    assert_eq!(d.fixes[0].edits, vec![(Span::new(0, 3), "foo".to_string())]);
    let mut d = Diagnostic::error(Code("E0201"), Span::new(0, 1), "x");
    d.fix("split", Span::new(0, 1), "a\nb");
    d.match_line_endings("x\r\ny\r\n");
    assert_eq!(d.fixes[0].edits[0].1, "a\r\nb");
}

#[test]
fn a_sink_keeps_one_diagnostic_per_place_and_none_inside_poison() {
    use cpl_diag::{Severity, Sink};
    let err = |a, b, code| Diagnostic::error(Code(code), Span::new(a, b), "x");
    let mut sink = Sink::with_poisoned(vec![Span::new(0, 10)]);
    assert!(sink.is_empty() && sink.last_mut().is_none());
    sink.push(err(2, 4, "E0201"));
    sink.push(err(12, 14, "E0201"));
    sink.push(err(12, 14, "E0201"));
    sink.push(err(12, 14, "E0301"));
    sink.push(Diagnostic::warning(Code("L0306"), Span::new(12, 14), "w"));
    sink.push(err(20, 22, "E0201"));
    let kept: Vec<_> = sink
        .diagnostics()
        .iter()
        .map(|d| (d.span.start, d.severity))
        .collect();
    assert_eq!(
        kept,
        [
            (12, Severity::Error),
            (12, Severity::Warning),
            (20, Severity::Error)
        ]
    );
    assert!(!sink.is_empty() && sink.len() == 3);
    assert_eq!(sink.last_mut().map(|d| d.span.start), Some(20));
    assert!(sink.is_poisoned(Span::new(3, 5)) && !sink.is_poisoned(Span::new(9, 12)));
    assert!(sink.encloses_poison(Span::new(0, 20)) && !sink.encloses_poison(Span::new(12, 20)));
    sink.poison(Span::new(30, 31));
    assert_eq!(sink.poisoned(), [Span::new(0, 10), Span::new(30, 31)]);
    sink.drop_where(|d| d.severity == Severity::Warning);
    assert_eq!(sink.len(), 2);
}

#[test]
fn doc_text_turns_math_to_unicode_and_links_to_code() {
    use cpl_diag::plain::{math, text};
    assert_eq!(
        text("costs \\( O(n \\log n) \\) per [`up`] and [`walk`](Rooted.up)."),
        "costs O(n log n) per `up` and `walk`."
    );
    assert_eq!(text("a \\( x"), "a \\( x");
    assert_eq!(text("[`a`](a-b) and [`b"), "`a`(a-b) and [`b");
    assert_eq!(text("\\(n^2\\) then [`c`]"), "n² then `c`");
    assert_eq!(math("n^2 + 2^{10} + x^{ab}"), "n² + 2¹⁰ + x^(ab)");
    assert_eq!(math("^{0123456789n}"), "⁰¹²³⁴⁵⁶⁷⁸⁹ⁿ");
    assert_eq!(
        math("\\sqrt{n} \\cdot \\mathit{cnt} \\le 10^9"),
        "√n · cnt ≤ 10⁹"
    );
    assert_eq!(math("\\sqrt n"), "√n");
    assert_eq!(math("O( n )"), "O(n)");
    assert_eq!(math("\\lvert S \\rvert"), "|S|");
}

#[test]
fn line_ranges_drop_the_line_break_and_utf16_columns_floor_to_a_character() {
    let crlf = "ab\r\ncd";
    assert_eq!(LineIndex::new(crlf).line_range(0, crlf), (0, 2));
    assert_eq!(LineIndex::new(crlf).line_range(1, crlf), (4, 6));
    let blank = "\nx";
    assert_eq!(LineIndex::new(blank).line_range(0, blank), (0, 0));
    let wide = "é😀x";
    let lines = LineIndex::new(wide);
    let col = |at| lines.utf16(at, wide).col;
    assert_eq!(
        [col(0), col(1), col(2), col(4), col(6), col(7)],
        [0, 0, 1, 1, 3, 4]
    );
}

#[test]
fn a_path_is_shown_as_the_os_spells_it_outside_windows() {
    use cpl_diag::Shown;
    assert_eq!(std::path::Path::new("a/b.cp").shown(), "a/b.cp");
    #[cfg(not(windows))]
    assert_eq!(std::path::Path::new(r"a\b.cp").shown(), r"a\b.cp");
}

#[test]
fn json_gives_the_witness_as_names_and_values() {
    let src = "let b: 0..=9 = read()\n";
    let d = Diagnostic::warning(Code("L0599"), Span::new(4, 5), "`b` can be 0")
        .with_witness(vec![("b".into(), 0), ("n".into(), 1 << 70)]);
    let v = json::diagnostic(&d, &LineIndex::new(src));
    assert_eq!(
        field(&v, "witness").to_string(),
        r#"[{"name":"b","value":0},{"name":"n","value":"1180591620717411303424"}]"#
    );
    let plain = Diagnostic::warning(Code("L0599"), Span::new(4, 5), "m");
    assert_eq!(
        field(&json::diagnostic(&plain, &LineIndex::new(src)), "witness").to_string(),
        "[]"
    );
}

use super::*;

fn fmt(src: &str) -> String {
    let out = format(src).unwrap_or_else(|e| panic!("{e}\n{src}"));
    let again = format(&out).unwrap_or_else(|e| panic!("second pass: {e}\n{out}"));
    assert_eq!(out, again, "formatting twice changed the result");
    out
}

#[test]
fn crate_is_named() {
    assert_eq!(NAME, "cpl-fmt");
}

/// The example in the README.
#[test]
fn formats_the_readme_example() {
    let src = "let a:[I]1..=1e9=read( )\nfor i in I :\n  best max=a[i]+1 // grow\n";
    let want = "let a: [I]1..=1e9 = read()\nfor i in I:\n    best max= a[i] + 1  // grow\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn puts_one_space_around_binary_operators() {
    assert_eq!(
        fmt("let x=a+b*c-d/e%f\n"),
        "let x = a + b * c - d / e % f\n"
    );
    assert_eq!(fmt("x+=1\ny   <<=  2\n"), "x += 1\ny <<= 2\n");
    assert_eq!(
        fmt("if a<=b and c!=d or not e:\n  f()\n"),
        "if a <= b and c != d or not e:\n    f()\n"
    );
    assert_eq!(fmt("print(answer()??-1)\n"), "print(answer() ?? -1)\n");
}

#[test]
fn keeps_unary_operators_tight() {
    assert_eq!(fmt("let x = - 1\n"), "let x = -1\n");
    assert_eq!(fmt("let y = a - - b\n"), "let y = a - -b\n");
    assert_eq!(fmt("f( - x, ~ y)\n"), "f(-x, ~y)\n");
    assert_eq!(fmt("return -1\n"), "return -1\n");
    assert_eq!(fmt("let r = lo..=-5\n"), "let r = lo..=-5\n");
}

#[test]
fn formats_types_without_inner_spaces() {
    assert_eq!(
        fmt("let a : [ I ]1..=1e9=read( )\n"),
        "let a: [I]1..=1e9 = read()\n"
    );
    assert_eq!(
        fmt("var adj:[V]List<V> =[]\n"),
        "var adj: [V]List<V> = []\n"
    );
    assert_eq!(
        fmt("var best: [Spare]?Int = none\n"),
        "var best: [Spare]?Int = none\n"
    );
    assert_eq!(
        fmt("fn span(u: ?Bar,d: Int)->?(Int,Int):\n    return none\n"),
        "fn span(u: ?Bar, d: Int) -> ?(Int, Int):\n    return none\n"
    );
    assert_eq!(
        fmt("let m: Map<Int,List<Int>> = f::<Int>()\n"),
        "let m: Map<Int, List<Int>> = f::<Int>()\n"
    );
    assert_eq!(
        fmt("let a: [I]-100..=100 = read()\n"),
        "let a: [I]-100..=100 = read()\n"
    );
    assert_eq!(
        fmt("let d = a[i]-1\nlet e = a[i] -1\n"),
        "let d = a[i] - 1\nlet e = a[i] - 1\n"
    );
    assert_eq!(
        fmt("fn peel<D>(start: [D]Bool)->List<D>:\n    return []\n"),
        "fn peel<D>(start: [D]Bool) -> List<D>:\n    return []\n"
    );
}

#[test]
fn tells_a_comparison_from_a_generic() {
    assert_eq!(
        fmt("if a<b and c>d:\n    pass\n"),
        "if a < b and c > d:\n    pass\n"
    );
    assert_eq!(fmt("if N < M:\n    pass\n"), "if N < M:\n    pass\n");
    assert_eq!(fmt("let s: Set<V> = x\n"), "let s: Set<V> = x\n");
}

#[test]
fn keeps_table_and_type_brackets_apart() {
    assert_eq!(
        fmt("let order = [I] i => a[i]\n"),
        "let order = [I] i => a[i]\n"
    );
    assert_eq!(
        fmt("let t = [I, J] (i, j) => i.idx\n"),
        "let t = [I, J] (i, j) => i.idx\n"
    );
    assert_eq!(fmt("let x = fs[i](y)[0]\n"), "let x = fs[i](y)[0]\n");
}

#[test]
fn keeps_max_assignment_together() {
    assert_eq!(
        fmt("best max= max(right,left)+1\n"),
        "best max= max(right, left) + 1\n"
    );
    assert_eq!(fmt("best[c] min= b\n"), "best[c] min= b\n");
    assert_eq!(fmt("let max = 3\n"), "let max = 3\n");
}

#[test]
fn handles_calls_fields_ranges_and_postfix() {
    assert_eq!(
        fmt("print( max( best ) ! , j . prev ! )\n"),
        "print(max(best)!, j.prev!)\n"
    );
    assert_eq!(
        fmt("for i in 0 .. n by 2:\n    a [ i ] . push( 1 )\n"),
        "for i in 0..n by 2:\n    a[i].push(1)\n"
    );
    assert_eq!(
        fmt("let s = xs[.. k] + xs[k ..]\n"),
        "let s = xs[..k] + xs[k..]\n"
    );
    assert_eq!(fmt("use  lib :: graph :: *\n"), "use lib::graph::*\n");
    assert_eq!(fmt("use std.seq.{ a,b }\n"), "use std.seq.{a, b}\n");
    assert_eq!(
        fmt("fn f() -> yields (v: Int, d: Int):\n    yield (v: 1, d: 2)\n"),
        "fn f() -> yields (v: Int, d: Int):\n    yield (v: 1, d: 2)\n"
    );
    assert_eq!(
        fmt("use std.seq.{a}\nlet x = {a: 1}\n"),
        "use std.seq.{a}\nlet x = { a: 1 }\n"
    );
    assert_eq!(
        fmt("if (a+b)>c: return (a,b)\n"),
        "if (a + b) > c: return (a, b)\n"
    );
}

#[test]
fn spaces_block_expressions_and_decorators() {
    assert_eq!(
        fmt("let lim = {let s = f(); s*2+1}\n"),
        "let lim = { let s = f(); s * 2 + 1 }\n"
    );
    assert_eq!(fmt("let e = {}\n"), "let e = {}\n");
    assert_eq!(
        fmt("@ memo\nfn f(x:Int)->Int=x\n"),
        "@memo\nfn f(x: Int) -> Int = x\n"
    );
    assert_eq!(
        fmt("@wire(\"?\")  @final fn ask(x: Int)\n"),
        "@wire(\"?\") @final fn ask(x: Int)\n"
    );
}

#[test]
fn indents_four_spaces_per_level() {
    let src = "fn f(n: Int) -> Int:\n  var s = 0\n  for i in 0..n:\n     if i % 2 == 0:\n        s += i\n  return s\n";
    let want = "fn f(n: Int) -> Int:\n    var s = 0\n    for i in 0..n:\n        if i % 2 == 0:\n            s += i\n    return s\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn keeps_one_blank_line_and_ends_with_one_newline() {
    assert_eq!(
        fmt("\n\nlet a = 1\n\n\n\nlet b = 2   \nlet c = 3\n\n\n"),
        "let a = 1\n\nlet b = 2\nlet c = 3\n"
    );
    assert_eq!(fmt("let a = 1"), "let a = 1\n");
    assert_eq!(fmt(""), "");
    assert_eq!(fmt("\n\n"), "");
}

#[test]
fn keeps_comments_where_they_are() {
    let src = "// header\n\n//second\nlet a = 1 // why\nfn f():\n    // inside\n    g()\n    // last in block\n// top level\nh()\n// end\n";
    let want = "// header\n\n//second\nlet a = 1  // why\nfn f():\n    // inside\n    g()\n    // last in block\n// top level\nh()\n// end\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn aligns_trailing_comments_of_consecutive_lines() {
    let src =
        "let n = read() // count\nlet total = 0 // sum so far\nlet z = 1\nlet w = 2 // alone\n";
    let want =
        "let n = read()  // count\nlet total = 0   // sum so far\nlet z = 1\nlet w = 2  // alone\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn joins_a_bracket_that_fits_on_one_line() {
    let src = "let xs = [\n    1,\n    2,\n    3,\n]\nf(\n  a,\n  b\n)\n";
    assert_eq!(fmt(src), "let xs = [1, 2, 3]\nf(a, b)\n");
}

#[test]
fn keeps_the_comma_of_a_one_element_tuple() {
    assert_eq!(fmt("let t = (1,)\n"), "let t = (1,)\n");
    assert_eq!(fmt("let t = (1, 2,)\n"), "let t = (1, 2)\n");
}

#[test]
fn breaks_a_long_bracket_one_element_per_line() {
    let src = "fn solve(first_argument: Int, second_argument: List<Int>, third_argument: [V]List<V>, fourth: Int) -> Int:\n    return 0\n";
    let want = "fn solve(\n    first_argument: Int,\n    second_argument: List<Int>,\n    third_argument: [V]List<V>,\n    fourth: Int,\n) -> Int:\n    return 0\n";
    assert_eq!(fmt(src), want);
    for line in fmt(src).lines() {
        assert!(line.len() <= MAX_WIDTH);
    }
}

#[test]
fn breaks_nested_brackets_only_as_far_as_needed() {
    let src = "    let table = make_table(first_dimension_size, second_dimension_size, combine(left_operand_value, right_operand_value, extra_operand_value, more), 0)\n";
    let src = format!("fn f():\n{src}");
    let want = "fn f():\n    let table = make_table(\n        first_dimension_size,\n        second_dimension_size,\n        combine(left_operand_value, right_operand_value, extra_operand_value, more),\n        0,\n    )\n";
    assert_eq!(fmt(&src), want);
}

#[test]
fn does_not_add_a_comma_to_a_comprehension() {
    let src = "let total = sum(weight_of_edge[e] * multiplier_for_edge[e] + offset_for_edge[e] for e in edges if is_kept[e] and not is_removed[e])\n";
    let want = "let total = sum(\n    weight_of_edge[e] * multiplier_for_edge[e] + offset_for_edge[e] for e in edges if is_kept[e] and not is_removed[e]\n)\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn leaves_a_long_line_with_no_bracket_alone() {
    let src = format!("let x = {}\n", vec!["long_name"; 14].join(" + "));
    assert_eq!(fmt(&src), src);
}

#[test]
fn keeps_comments_inside_brackets() {
    let src = "let xs = [\n    1, // one\n    // about two\n    2,\n    3 // three\n]\n";
    let want = "let xs = [\n    1,  // one\n    // about two\n    2,\n    3,  // three\n]\n";
    assert_eq!(fmt(src), want);
    let src = "f(a, // first\n  b)\n";
    assert_eq!(fmt(src), "f(\n    a,  // first\n    b,\n)\n");
    let src = "f(\n    a\n    // closing note\n)\n";
    assert_eq!(fmt(src), "f(\n    a\n    // closing note\n)\n");
}

#[test]
fn never_fuses_two_tokens() {
    assert_eq!(fmt("let x = a ? ? b\n"), "let x = a ? ?b\n");
    assert_eq!(fmt("let y = r \"s\"\n"), "let y = r \"s\"\n");
}

#[test]
fn refuses_a_file_the_lexer_cannot_read() {
    let err = format("let x = f(1, 2\nlet y = 3\n").unwrap_err();
    assert!(
        matches!(err, FormatError::Lex(ref d) if d.len() == 1),
        "{err:?}"
    );
    assert!(matches!(format("if x:\n\ty\n"), Err(FormatError::Lex(_))));
    assert!(!is_formatted("let x = f(1, 2\n"));
}

#[test]
fn formats_code_the_parser_rejects() {
    assert_eq!(
        fmt("let (l, r) = read()   where l<=r\n"),
        "let (l, r) = read() where l <= r\n"
    );
    assert_eq!(fmt("x = = )\n"), "x = =)\n");
}

#[test]
fn is_formatted_agrees_with_format() {
    assert!(is_formatted("let x = 1\n"));
    assert!(!is_formatted("let x=1\n"));
}

/// Strip byte-offset `Span { start: ..., end: ... }` fields from `Debug` output
/// so two parsed ASTs can be compared for structural equivalence.
fn strip_spans(debug: &str) -> String {
    let mut out = String::with_capacity(debug.len());
    let mut rest = debug;
    while let Some(pos) = rest.find("Span {") {
        out.push_str(&rest[..pos]);
        out.push_str("Span");
        rest = &rest[pos + "Span {".len()..];
        if let Some(end) = rest.find('}') {
            rest = &rest[end + 1..];
        }
    }
    out.push_str(rest);
    out
}

/// Every `.cp` file in the repository, at any depth under `proto`,
/// `examples` and `stdlib`: formatting keeps the tokens (checked inside
/// `format`), is stable, and leaves the parser's verdict unchanged.
#[test]
fn formats_the_whole_corpus() {
    fn walk(dir: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, files);
            } else if path.extension().is_some_and(|e| e == "cp") {
                files.push(path);
            }
        }
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut files = Vec::new();
    for dir in ["proto", "examples", "stdlib"] {
        walk(&root.join(dir), &mut files);
    }
    assert!(files.len() >= 1000, "found only {} corpus files", files.len());
    let mut formatted = 0;
    for file in &files {
        let src = std::fs::read_to_string(file).unwrap();
        let out = match format(&src) {
            Ok(out) => out,
            Err(FormatError::Lex(_)) => continue,
            Err(e) => panic!("{}: {e}", file.display()),
        };
        formatted += 1;
        assert_eq!(
            format(&out).as_ref(),
            Ok(&out),
            "{} is not stable",
            file.display()
        );
        let p_src = cpl_syntax::parse_file(&src);
        let p_out = cpl_syntax::parse_file(&out);
        assert_eq!(
            p_src.diagnostics.len(),
            p_out.diagnostics.len(),
            "{}: the parser reads it differently",
            file.display()
        );
        if p_src.diagnostics.is_empty() {
            let ast = |p: &cpl_syntax::ParsedFile| {
                let items: Vec<_> = p.items.iter().map(|(_, item)| item).collect();
                strip_spans(&format!("{items:?}"))
            };
            assert_eq!(
                ast(&p_src),
                ast(&p_out),
                "{}: formatting changed the parsed AST",
                file.display()
            );
        }
        for line in out.lines() {
            assert_eq!(line, line.trim_end(), "{}", file.display());
        }
    }
    assert!(formatted >= 1000, "only {formatted} files formatted");
}

mod cli {
    use crate::cli::run;
    use std::path::PathBuf;

    /// A fresh directory under the system's temporary one.
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cpl-fmt-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn fmt(args: &[&str]) -> (u8, String, String) {
        let args: Vec<String> = args.iter().map(|a| a.to_string()).collect();
        let (mut out, mut err) = (Vec::new(), Vec::new());
        let code = run(&args, &mut out, &mut err);
        (
            code,
            String::from_utf8(out).unwrap(),
            String::from_utf8(err).unwrap(),
        )
    }

    #[test]
    fn rewrites_files_under_a_directory() {
        let dir = scratch("rewrite");
        std::fs::create_dir_all(dir.join("lib")).unwrap();
        std::fs::create_dir_all(dir.join("target")).unwrap();
        std::fs::write(dir.join("a.cp"), "let x=1\n").unwrap();
        std::fs::write(dir.join("lib/b.cp"), "let y = 2\n").unwrap();
        std::fs::write(dir.join("target/c.cp"), "let z=3\n").unwrap();
        std::fs::write(dir.join("notes.txt"), "let w=4\n").unwrap();

        let (code, out, err) = fmt(&[dir.to_str().unwrap()]);
        assert_eq!(code, 0, "{err}");
        assert!(out.contains("formatted") && out.contains("a.cp"), "{out}");
        assert!(!out.contains("b.cp"), "{out}");
        assert!(err.contains("2 files checked, 1 changed"), "{err}");
        assert_eq!(
            std::fs::read_to_string(dir.join("a.cp")).unwrap(),
            "let x = 1\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("target/c.cp")).unwrap(),
            "let z=3\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("notes.txt")).unwrap(),
            "let w=4\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn check_lists_files_and_changes_nothing() {
        let dir = scratch("check");
        let file = dir.join("a.cp");
        std::fs::write(&file, "let x=1\n").unwrap();

        let (code, out, err) = fmt(&["--check", file.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(out.starts_with("would reformat "), "{out}");
        assert!(err.contains("1 file checked, 1 would change"), "{err}");
        assert_eq!(std::fs::read_to_string(&file).unwrap(), "let x=1\n");

        std::fs::write(&file, "let x = 1\n").unwrap();
        let (code, out, _) = fmt(&["--check", file.to_str().unwrap()]);
        assert_eq!((code, out.as_str()), (0, ""));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn reports_a_file_it_cannot_format_and_goes_on() {
        let dir = scratch("broken");
        std::fs::write(dir.join("a.cp"), "let x = f(1\n").unwrap();
        std::fs::write(dir.join("b.cp"), "let y=2\n").unwrap();

        let (code, _, err) = fmt(&[dir.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(err.contains("a.cp: not formatted"), "{err}");
        assert!(err.contains("1 not formatted"), "{err}");
        assert_eq!(
            std::fs::read_to_string(dir.join("a.cp")).unwrap(),
            "let x = f(1\n"
        );
        assert_eq!(
            std::fs::read_to_string(dir.join("b.cp")).unwrap(),
            "let y = 2\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn rejects_unknown_options_and_missing_files() {
        let (code, _, err) = fmt(&["--fast"]);
        assert_eq!(code, 2);
        assert!(err.contains("unknown option `--fast`"), "{err}");
        let (code, _, err) = fmt(&["/no/such/file.cp"]);
        assert_eq!(code, 1);
        assert!(err.contains("can't read /no/such/file.cp"), "{err}");
        let (code, out, _) = fmt(&["--help"]);
        assert_eq!(code, 0);
        assert!(out.starts_with("usage: cpl fmt"), "{out}");
    }
}

#[test]
fn keeps_an_inner_attribute_tight() {
    assert_eq!(fmt("@! multitest\nprint(1)\n"), "@!multitest\nprint(1)\n");
    assert_eq!(fmt("@!multitest\nprint(1)\n"), "@!multitest\nprint(1)\n");
    assert_eq!(fmt("@! interactive\n@!multitest\nprint(1)\n"), "@!interactive\n@!multitest\nprint(1)\n");
    assert_eq!(fmt("@ memo\nfn f(n: Int) -> Int = n\n"), "@memo\nfn f(n: Int) -> Int = n\n");
    assert_eq!(fmt("let y = o ! + 1\n"), "let y = o! + 1\n");
}

#[test]
fn formats_indented_lambda_blocks() {
    let src = "let lv = [0..=s, 0..n] (i, j) =>\n  let miss = if j > 0: a else: b\n  p * lv[i, j] + miss\nxs.sort(by: x =>\n  let d = x % 10\n  d\n)\n";
    let want = "let lv = [0..=s, 0..n] (i, j) =>\n    let miss = if j > 0: a else: b\n    p * lv[i, j] + miss\nxs.sort(by: x =>\n    let d = x % 10\n    d\n)\n";
    assert_eq!(fmt(src), want);
    assert_eq!(fmt(want), want);
}

#[test]
fn formats_trailing_binary_operator_continuations() {
    let src = "if a == 1 and\n    b == 2:\n    f()\n";
    assert_eq!(fmt(src), "if a == 1 and b == 2:\n    f()\n");

    let with_comment = "if a == 1 and  // check\n    b == 2:\n    f()\n";
    assert_eq!(fmt(with_comment), with_comment);
}

#[test]
fn formats_punned_named_args_and_records() {
    let src = "let s = State(moves : , cost : , step : )\nlet r = (val : , prev : )\n";
    let want = "let s = State(moves:, cost:, step:)\nlet r = (val:, prev:)\n";
    assert_eq!(fmt(src), want);
    assert_eq!(fmt(want), want);
}

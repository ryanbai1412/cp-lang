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
    assert_eq!(fmt("print(t.0, o!.1)\n"), "print(t.0, o!.1)\n");
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
fn keeps_a_bang_in_typo_one_operator() {
    assert_eq!(fmt("print(s !in reach)\n"), "print(s !in reach)\n");
    assert_eq!(fmt("print(s!.len)\n"), "print(s!.len)\n");
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
fn breaks_a_long_operator_chain_after_its_loosest_operators() {
    let src = format!("let x = {}\n", ["long_name"; 14].join(" + "));
    let want = format!("let x = {} +\n", ["long_name"; 14].join(" +\n    "));
    let want = want.trim_end_matches(" +\n").to_string() + "\n";
    assert_eq!(fmt(&src), want);
    let src = "psum[(i + 1) * w + (j + 1)] = flat[i * c + j] + psum[(i + 1) * w + j] + psum[i * w + (j + 1)] - psum[i * w + j]\n";
    let want = "psum[(i + 1) * w + (j + 1)] = flat[i * c + j] +\n    psum[(i + 1) * w + j] +\n    psum[i * w + (j + 1)] -\n    psum[i * w + j]\n";
    assert_eq!(fmt(src), want);
    assert_eq!(fmt(want), want);
    let src = format!(
        "if {} or ready:\n    go()\n",
        ["long_flag_name"; 5].join(" and ")
    );
    let want = format!(
        "if {} or\n        ready:\n    go()\n",
        ["long_flag_name"; 5].join(" and ")
    );
    assert_eq!(fmt(&src), want);
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
    for dir in [
        "corpus/problems",
        "corpus/lib",
        "crates/cpl-cli/tests/cases",
        "examples",
    ] {
        walk(&root.join(dir), &mut files);
    }
    assert!(
        files.len() >= 1000,
        "found only {} corpus files",
        files.len()
    );
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
            // The same items, except that each run of `use` items may be
            // reordered.
            let ast = |p: &cpl_syntax::ParsedFile| {
                let mut items: Vec<String> = p
                    .items
                    .iter()
                    .map(|(_, item)| strip_spans(&format!("{item:?}")))
                    .collect();
                let is_use = |s: &String| s.starts_with("ParsedItem { stmts: [Stmt { kind: Use {");
                let mut i = 0;
                while i < items.len() {
                    let j = i + items[i..].iter().take_while(|s| is_use(s)).count();
                    items[i..j].sort();
                    i = j.max(i + 1);
                }
                items
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
    assert_eq!(
        fmt("@! interactive\n@!multitest\nprint(1)\n"),
        "@!interactive\n@!multitest\nprint(1)\n"
    );
    assert_eq!(
        fmt("@ memo\nfn f(n: Int) -> Int = n\n"),
        "@memo\nfn f(n: Int) -> Int = n\n"
    );
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

#[test]
fn keeps_an_argument_after_a_block_lambda_outside_the_block() {
    let src = "let t = f(x =>\n    let y = x\n    y\n, 3)\n";
    assert_eq!(format(src).unwrap(), src);
}

#[test]
fn formats_loop_else() {
    let src = "for x in xs:\n  if x == 0:\n    break\nelse:\n  print(-1)\n";
    let want = "for x in xs:\n    if x == 0:\n        break\nelse:\n    print(-1)\n";
    assert_eq!(fmt(src), want);
    assert_eq!(fmt(want), want);
}

#[test]
fn formats_context_with_and_fn_uses() {
    let src = "context opt:\n  k: Int = 2\nfn pick() -> Int uses opt = opt.k\nwith io(base: 1)\nwith io(sep: \",\"):\n  print(pick())\n";
    let want = "context opt:\n    k: Int = 2\nfn pick() -> Int uses opt = opt.k\nwith io(base: 1)\nwith io(sep: \",\"):\n    print(pick())\n";
    assert_eq!(fmt(src), want);
    assert_eq!(fmt(want), want);
}

#[test]
fn keeps_the_c_spellings_the_parser_explains_tight() {
    // The parser explains `&&`, `||` and `..<` only when their halves
    // touch, and `!x` only before a value; spacing them apart would change
    // the error (crates/cpl-cli/tests/fixes/c_operators.cp).
    for src in [
        "if n > 1 && ok:\n    x\n",
        "if !ok || n == 5:\n    x\n",
        "for i in 0..<n:\n    x\n",
    ] {
        assert_eq!(fmt(src), src);
    }
    assert_eq!(fmt("if n>1&&ok:\n    x\n"), "if n > 1 && ok:\n    x\n");
    assert_eq!(
        fmt("let a = f(!x, y!)\nlet b = a! + 1\n"),
        "let a = f(!x, y!)\nlet b = a! + 1\n"
    );
}

#[test]
fn spaces_impl_generics_lends_and_without_like_the_spec() {
    assert_eq!(
        fmt("impl<T: Total> Total for ?T:\n    x\n"),
        "impl<T: Total> Total for ?T:\n    x\n"
    );
    assert_eq!(
        fmt("fn f(a: X, b: X) -> lends (a) X = b\n"),
        "fn f(a: X, b: X) -> lends(a) X = b\n"
    );
    assert_eq!(
        fmt("fn s<R: (u: Int, ..)>(r: R) -> (..R without(u)) = (..r without(u))\n"),
        "fn s<R: (u: Int, ..)>(r: R) -> (..R without (u)) = (..r without (u))\n"
    );
    assert_eq!(fmt("let k = without(3)\n"), "let k = without(3)\n");
}

#[test]
fn spaces_shorthands_reads_and_member_keywords_like_the_spec() {
    let cases = [
        (
            "let e = es.sorted(by:.len)\n",
            "let e = es.sorted(by: .len)\n",
        ),
        ("let f = _ =>.empty\n", "let f = _ => .empty\n"),
        ("let g = f(.x)\n", "let g = f(.x)\n"),
        (
            "let (n, k) = read (Int, Int)\n",
            "let (n, k) = read (Int, Int)\n",
        ),
        ("let n = read(1..=5000)\n", "let n = read(1..=5000)\n"),
        ("let a = read[n]Int\n", "let a = read [n]Int\n"),
        ("let a = read()\n", "let a = read()\n"),
        ("let a = self.read(x)\n", "let a = self.read(x)\n"),
        ("fn read(x: Int) = x\n", "fn read(x: Int) = x\n"),
        ("let x = sat.var () + 1\n", "let x = sat.var() + 1\n"),
        ("print(- - 4, 5 - -3)\n", "print(- -4, 5 - -3)\n"),
        ("var b: ?? Int = none\n", "var b: ??Int = none\n"),
        ("let c = a ?? b\n", "let c = a ?? b\n"),
        ("print((s !in reach).int)\n", "print((s !in reach).int)\n"),
        ("out.push(f\"{x} \")\n", "out.push(f\"{x} \")\n"),
    ];
    for (src, want) in cases {
        assert_eq!(fmt(src), want, "{src:?}");
    }
}

#[test]
fn sorts_a_run_of_use_lines_std_first() {
    let src = "use lib.flow as fl\nuse std.segtree\nuse std.graph.{dijkstra, Paths}\nuse lib.treap\nprint(1)\n";
    let want = "use std.graph.{dijkstra, Paths}\nuse std.segtree\nuse lib.flow as fl\nuse lib.treap\nprint(1)\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn keeps_use_groups_and_trailing_comments() {
    let src = "use std.seq  // sort\nuse std.dsu\n\nuse lib.b\n// local\nuse lib.a\n";
    let want = "use std.dsu\nuse std.seq  // sort\n\nuse lib.b\n// local\nuse lib.a\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn leaves_a_run_with_a_multi_line_use_alone() {
    let src = "use std.seq\nuse std.graph.{\n    long_function_name_0,\n    long_function_name_1,\n    long_function_name_2,\n    long_function_name_3,\n    long_function_name_4,\n    long_function_name_5,\n}\nuse std.dsu\n";
    let out = fmt(src);
    assert!(out.starts_with("use std.seq\n"), "{out}");
    assert!(out.ends_with("use std.dsu\n"), "{out}");
}

#[test]
fn sorts_use_lines_inside_a_block() {
    let src = "fn f():\n    use std.seq\n    use std.dsu\n    print(1)\n";
    let want = "fn f():\n    use std.dsu\n    use std.seq\n    print(1)\n";
    assert_eq!(fmt(src), want);
}

#[test]
fn cp_toml_sets_the_indent_and_line_widths() {
    let (style, errors) = Style::from_toml(
        "[lints]\nindent_width = 9\n\n[fmt]\nindent_width = 2  # narrow\nline_width = 60\n",
    );
    assert_eq!(errors, []);
    assert_eq!(
        style,
        Style {
            indent: 2,
            line_width: 60
        }
    );
    let (style, errors) =
        Style::from_toml("[fmt]\nindent_width = 1\ntabs = true\nline_width = 80\n");
    let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 3]);
    assert_eq!(
        errors[0].to_string(),
        "line 2: `indent_width` is a number from 2 to 8, not `1`"
    );
    // The good line still applies.
    assert_eq!(style.line_width, 80);
    assert_eq!(style.indent, INDENT);
}

#[test]
fn a_style_changes_only_the_widths() {
    let src = "fn f(a: Int) -> Int:\n    if a > 0:\n        return g(alpha_value_one, beta_value_two, gamma_value_three)\n    a\n";
    let narrow = Style {
        indent: 2,
        line_width: 40,
    };
    let out = format_with(src, &narrow).unwrap();
    assert_eq!(
        out,
        "fn f(a: Int) -> Int:\n  if a > 0:\n    return g(\n      alpha_value_one,\n      beta_value_two,\n      gamma_value_three,\n    )\n  a\n"
    );
    assert_eq!(format_with(&out, &narrow).unwrap(), out);
    // Back to the default style gives the default layout.
    assert_eq!(format(&out).unwrap(), fmt(src));
    assert!(cpl_syntax::parse_file(&out).diagnostics.is_empty());
}

#[test]
fn keeps_crlf_line_endings_of_a_windows_checkout() {
    let out = fmt("let  x=1\r\nfn f():\r\n    return  x\r\nprint( f() )\r\n");
    assert_eq!(
        out,
        "let x = 1\r\nfn f():\r\n    return x\r\nprint(f())\r\n"
    );
    assert!(crate::is_formatted(&out));
    // Mixed endings are not a Windows checkout: they become LF.
    assert_eq!(fmt("let  x=1\r\nprint( x )\n"), "let x = 1\nprint(x)\n");
}

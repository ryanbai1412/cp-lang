use super::*;

/// The codes `src` is reported for, in source order.
fn codes(src: &str) -> Vec<&'static str> {
    codes_with(src, &Config::default())
}

fn codes_with(src: &str, config: &Config) -> Vec<&'static str> {
    lint(src, config).iter().map(|d| d.code.0).collect()
}

fn only(src: &str) -> Diagnostic {
    let mut found = lint(src, &Config::default());
    assert_eq!(found.len(), 1, "{found:#?}");
    found.remove(0)
}

#[test]
fn crate_is_named() {
    assert_eq!(NAME, "cpl-lint");
}

#[test]
fn every_lint_has_its_own_code_and_name() {
    for (i, a) in LINTS.iter().enumerate() {
        assert!(a.code.0.starts_with('L') && a.code.0.len() == 5, "{a:?}");
        assert_eq!(lint_named(a.name), Some(a));
        for b in &LINTS[i + 1..] {
            assert_ne!(a.code, b.code);
            assert_ne!(a.name, b.name);
        }
    }
}

#[test]
fn clean_code_gets_nothing() {
    let src = "let n: 1..=2e5 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nvar best = 0\nfor v in V:\n    best max= a[v]\nprint(best)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_file_with_a_syntax_error_gets_nothing() {
    assert_eq!(codes("var x = (1\nprint(x)\n"), Vec::<&str>::new());
}

#[test]
fn var_never_reassigned_offers_let() {
    let src = "var total = 0\nprint(total)\n";
    let found = only(src);
    assert_eq!(found.code, VAR_NEVER_REASSIGNED.code);
    assert_eq!(found.message, "`total` is declared `var` but never changed");
    assert_eq!(found.span.text(src), "var");
    let (fixed, applied) = apply_fixes(src, &[found]);
    assert_eq!(
        (fixed.as_str(), applied),
        ("let total = 0\nprint(total)\n", 1)
    );
    assert_eq!(codes(&fixed), Vec::<&str>::new());
}

#[test]
fn every_way_of_changing_a_var_counts() {
    for change in [
        "x = 1",
        "x += 1",
        "x max= 1",
        "x[0] = 1",
        "x.w = 1",
        "x.push(1)",
        "fill(mut x)",
        "for i in 0..3:\n    x += i",
        "fn bump():\n    x += 1",
        "let f = y => { x += y; 0 }",
    ] {
        let src = format!("var x = make()\n{change}\nprint(x)\n");
        assert_eq!(codes(&src), Vec::<&str>::new(), "{change}");
    }
}

#[test]
fn a_var_is_judged_by_its_own_scope() {
    // The inner `x` changes; the outer one never does.
    let src = "var x = 0\nfn f():\n    var y = 0\n    y += 1\n    print(y)\nprint(x)\n";
    assert_eq!(codes(src), ["L0201"]);
    // A tuple pattern stays `var` if either name changes.
    assert_eq!(
        codes("var (a, b) = read()\na += 1\nprint(a, b)\n"),
        Vec::<&str>::new()
    );
    // So does one assigned as a whole tuple, and a lone `var` assigned inside one.
    assert_eq!(
        codes("var (x, y) = read()\nwhile y != 0: (x, y) = (y, x % y)\nprint(x)\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("var x = 1\nvar y = 2\n(x, y) = (y, x)\nprint(x, y)\n"),
        Vec::<&str>::new()
    );
    let both = only("var (a, b) = read()\nprint(a, b)\n");
    assert_eq!(
        both.message,
        "`a` and `b` are declared `var` but never changed"
    );
}

#[test]
fn shadowing_a_top_level_let_is_reported_once_per_binding() {
    let src = "let n: 1..=9 = read()\nfn f(k: Int) -> Int:\n    let n = k + 1\n    n\nprint(f(n))\n";
    let found = only(src);
    assert_eq!(found.code, SHADOWS_TOP_LEVEL.code);
    assert_eq!(found.message, "this local hides the top-level `n`");
    assert_eq!(found.labels[0].message, "`n` is defined here, on line 1");
    // A top-level fn names its own parameters: `fn sub(a: Pt, b: Pt)`
    // below a `let a` hides nothing. A fn inside a fn still does.
    assert_eq!(
        codes("let a: Int = read()\nfn sub(a: Int, b: Int) -> Int = a - b\nprint(sub(a, 1))\n"),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("let a: Int = read()\nfn g() -> Int:\n    fn sub(a: Int) -> Int = a - 1\n    sub(3)\nprint(g() + a)\n"),
        ["L0203"]
    );
    // Loop variables and locals count; `_` and locals hiding locals do not.
    assert_eq!(
        codes("let i = 1\nfor i in 0..3:\n    print(i)\nprint(i)\n"),
        ["L0203"]
    );
    let quiet = "fn f():\n    let x = 1\n    for _ in 0..3:\n        let x = 2\n        print(x)\n    print(x)\nf()\n";
    assert_eq!(codes(quiet), Vec::<&str>::new());
    // A fn written above the `let` it would hide.
    assert_eq!(
        codes("fn f():\n    let k = 1\n    print(k)\nlet k = 2\nprint(k)\nf()\n"),
        ["L0203"]
    );
}

#[test]
fn domain_keyed_set_is_left_alone_when_sparse_or_ordered() {
    let head = "let n: 1..=9 = read()\ndomain V = 1..=n\n";
    // Plain membership: the array of `Bool` is the better shape.
    let dense = format!("{head}var seen: Set<V> = Set<V>()\nseen.insert(V.first)\nprint(V.first in seen)\n");
    assert_eq!(codes(&dense), ["L0412"]);
    // A set per `V` holds few of them; `[V][V]Bool` would hold all.
    let rows = format!("{head}var adj: [V]Set<V> = Set<V>()\nadj[V.first].insert(V.last)\nprint(adj[V.first].len)\n");
    assert_eq!(codes(&rows), Vec::<&str>::new());
    let listed = format!("{head}let groups: List<Set<V>> = []\nprint(groups.len)\n");
    assert_eq!(codes(&listed), Vec::<&str>::new());
    // Asked for its smallest or largest, which an array cannot answer.
    for ask in ["free.first", "free.last", "free.pop_first()", "free.pop_last()", "free.first_ge(V.first)"] {
        let src = format!("{head}var free: Set<V> = Set<V>()\nfree.insert(V.first)\nprint({ask})\n");
        assert_eq!(codes(&src), Vec::<&str>::new(), "{ask}");
    }
    let inferred = format!("{head}var free = Set<V>()\nfree.insert(V.first)\nprint(free.first)\n");
    assert_eq!(codes(&inferred), Vec::<&str>::new());
}

#[test]
fn domain_keyed_map_names_the_array_to_use() {
    let src = "let n: 1..=9 = read()\ndomain V = 1..=n\nvar cnt: Map<V, Int> = Map<V, Int>()\ncnt[V.first] = 1\nprint(cnt)\n";
    let found = only(src);
    assert_eq!(found.code.0, "L0412");
    assert_eq!(found.message, "`Map<V, Int>` is keyed by the domain `V`");
    assert_eq!(
        found.help.as_deref(),
        Some("`[V]?Int` uses one slot per `V` and has no lookup cost")
    );
    let set = only("domain V = 1..=5\nvar seen = Set<V>()\nseen.insert(V.first)\nprint(seen)\n");
    assert_eq!(
        set.help.as_deref(),
        Some("`[V]Bool` uses one slot per `V` and has no lookup cost")
    );
    // A map keyed by a number is what a map is for.
    assert_eq!(
        codes("var m: Map<Int, Int> = Map<Int, Int>()\nm[3] = 1\nprint(m)\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn position_arithmetic_points_at_next_prev_and_offset() {
    let wrap = |call: &str| format!("domain V = 1..=5\nlet v = V.first\nprint({call})\n");
    let next = only(&wrap("V(v.ord + 1)"));
    assert_eq!(next.code, POSITION_ARITHMETIC.code);
    assert_eq!(next.message, "`V(v.ord + 1)` builds a `V` from a position");
    assert!(next
        .help
        .as_deref()
        .unwrap()
        .starts_with("`v.next` is the `V` after `v`"));
    let prev = only(&wrap("V(v.idx - 1)"));
    assert!(prev
        .help
        .as_deref()
        .unwrap()
        .starts_with("`v.prev` is the `V` before `v`"));
    let far = only(&wrap("V(2 + v.ord)"));
    assert!(far
        .help
        .as_deref()
        .unwrap()
        .starts_with("`v.offset(2)` moves `v` by `2`"));
    // A label is a number already, and `f` is not a domain.
    assert_eq!(codes(&wrap("V(3)")), Vec::<&str>::new());
    assert_eq!(codes(&wrap("f(v.ord + 1)")), Vec::<&str>::new());
}

#[test]
fn float_equality_follows_floats_through_names() {
    let found = only("let x: F64 = read()\nif x == 0.5:\n    print(1)\n");
    assert_eq!(found.code, FLOAT_EQUALITY.code);
    assert!(found
        .help
        .as_deref()
        .unwrap()
        .contains("`abs(x - 0.5) < EPS`"));
    // Through an inferred `let`, a fn result, a field and arithmetic.
    assert_eq!(
        codes("let half = 1.0 / 2.0\nlet twice = half * 2\nprint(twice != 1)\n"),
        ["L0301"]
    );
    assert_eq!(
        codes("fn area(r: Int) -> F64 = 3.14 * r * r\nprint(area(2) == area(3))\n"),
        ["L0301"]
    );
    assert_eq!(
        codes("struct P:\n    x: F64\nlet p: P = read()\nprint(p.x == 0)\n"),
        ["L0301"]
    );
    assert_eq!(codes("let d = sqrt(read())\nprint(d == 2)\n"), ["L0301"]);
    // Ordering floats is fine, and so is `==` on integers.
    assert_eq!(
        codes("let x: F64 = read()\nprint(x < 0.5, 1 == 2)\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn unread_field_ignores_writes_and_libraries() {
    let program =
        "struct Edge:\n    to: Int\n    id: Int\nvar e: Edge = read()\ne.id = 7\nprint(e.to)\n";
    let found = only(program);
    assert_eq!(found.code, UNREAD_FIELD.code);
    assert_eq!(found.message, "field `id` of `Edge` is never read");
    // `e.id += 1` reads it; a method reading `self.id` reads it.
    assert_eq!(
        codes(&program.replace("e.id = 7", "e.id += 7")),
        Vec::<&str>::new()
    );
    let method = "struct Edge:\n    id: Int\n    fn key(self) -> Int = self.id\nlet e: Edge = read()\nprint(e.key())\n";
    assert_eq!(codes(method), Vec::<&str>::new());
    // A file of declarations is a library: its users read the fields.
    assert_eq!(
        codes("struct Edge:\n    to: Int\n    id: Int\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn memo_key_space_multiplies_the_input_bounds() {
    let src = "let n: 1..=2e5 = read()\ndomain V = 1..=n\n@memo\nfn go(a: V, b: V) -> Int:\n    0\nprint(go(V.first, V.first))\n";
    let found = only(src);
    assert_eq!(found.code.0, "L0413");
    assert_eq!(
        found.message,
        "`@memo` on `go` can hold 4.0e10 entries, at least 298 GB of memory; the limit is 256 MB"
    );
    assert_eq!(
        found.notes,
        ["one entry per combination of arguments: 200000 for `a` x 200000 for `b`"]
    );
    // 200000 entries fit; so does the pair once the limit is raised.
    assert_eq!(
        codes(&src.replace("a: V, b: V", "a: V")),
        Vec::<&str>::new()
    );
    let (roomy, errors) = Config::from_toml("[lints]\nmemory_mb = 400000\n");
    assert!(errors.is_empty());
    assert_eq!(codes_with(src, &roomy), Vec::<&str>::new());
    // A hashed memo stores only the states it reaches.
    assert_eq!(
        codes(&src.replace("@memo", "@memo(hash)")),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes("@memo\nfn steps(x: 1..=1e18) -> Int:\n    0\nprint(steps(5))\n"),
        Vec::<&str>::new()
    );
    // Bounds this cannot see through say nothing.
    assert_eq!(
        codes("let n = read()\ndomain V = 1..=n\n@memo\nfn go(a: V, b: V) -> Int:\n    0\nprint(go(V.first, V.first))\n"),
        Vec::<&str>::new()
    );
}

#[test]
fn bounds_follow_consts_and_arithmetic() {
    let src = "const K = 2000\nlet n: 1..=K * K = read()\ndomain Cell = 0..n\ndomain Bit = 0..2\n@memo\nfn f(c: Cell, d: Cell, b: Bit) -> Int:\n    0\nprint(f(Cell.first, Cell.first, Bit.first))\n";
    let found = only(src);
    assert_eq!(
        found.notes,
        ["one entry per combination of arguments: 4000000 for `c` x 4000000 for `d` x 2 for `b`"]
    );
}

#[test]
fn unproven_list_index_wants_two_loops_and_an_unproven_index() {
    let nest = |index: &str| {
        format!("var ws: List<Int> = []\nws.push(read())\nvar total = 0\nfor i in 0..ws.len:\n    for j in ws.indices:\n        total += {index}\nprint(total)\n")
    };
    let found = only(&nest("ws[j + 1]"));
    assert_eq!(found.code.0, "L0414");
    assert_eq!(
        found.message,
        "`ws[j + 1]` is bounds-checked on every pass of a nested loop"
    );
    // Loop variables over the list's own indices are proven.
    assert_eq!(codes(&nest("ws[i] * ws[j]")), Vec::<&str>::new());
    // One loop is not hot, and an array indexed by a domain is not a list.
    let single = "var ws: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    print(ws[i + 1])\n";
    assert_eq!(codes(single), Vec::<&str>::new());
    assert_eq!(codes(&nest("ws[i..j].len")), Vec::<&str>::new());
    // A name is judged by its nearest binding: a list in one fn does not make
    // an array of the same name in another a list.
    let shadowed = "fn f(seen: List<Bool>) -> Bool = seen[0]\ndomain V = 0..9\nvar seen: [V]Bool = false\nfor u in V:\n    for v in V:\n        seen[v] = seen[u]\nprint(f([true]))\n";
    assert!(!codes(shadowed).contains(&"L0414"));
    let inner = "domain V = 0..9\nvar ws: [V]Int = 0\nfn g():\n    var ws: List<Int> = []\n    ws.push(read())\n    for i in 0..3:\n        for j in 0..3:\n            print(ws[i + j])\ng()\nprint(ws)\n";
    assert!(codes(inner).contains(&"L0414"));
    // A fn starts its own count.
    let in_fn = "var ws: List<Int> = []\nws.push(read())\nfor i in 0..3:\n    for j in 0..3:\n        let f = k => ws[k]\n        print(f(i + j))\n";
    assert_eq!(codes(in_fn), ["L0414"]);
    // `while head < q.len` is the check, for a list that grows as it is read.
    let worklist = "var q: List<Int> = [0]\nfor root in 0..3:\n    var head = 0\n    while head < q.len:\n        let u = q[head]\n        head += 1\n        if u < 3: q.push(u + 1)\n";
    assert_eq!(codes(worklist), Vec::<&str>::new());
    // A `@test` is not what a judge times; the code after it is.
    let test = "@test\nfn t():\n    let ws = [1, 2, 3]\n    for i in 0..2:\n        for j in 0..2:\n            assert ws[i + j] > 0\n";
    assert_eq!(codes(test), Vec::<&str>::new());
    let after = format!("{test}fn f(ws: List<Int>):\n    for i in 0..2:\n        for j in 0..2:\n            print(ws[i + j])\nf([1, 2, 3])\n");
    assert_eq!(codes(&after), ["L0414"]);
}

#[test]
fn allow_covers_the_item_it_decorates_and_nothing_else() {
    let src = "@allow(var_never_reassigned)\nvar a = 1\nvar b = 2\nprint(a, b)\n";
    let found = only(src);
    assert_eq!(found.span.text(src), "var");
    assert_eq!(found.message, "`b` is declared `var` but never changed");
    // On a fn it covers the whole body.
    let body = "let n = 1\n@allow(shadows_top_level)\nfn f(n: Int):\n    for n in 0..3:\n        print(n)\nf(n)\n";
    assert_eq!(codes(body), Vec::<&str>::new());
}

#[test]
fn an_unknown_lint_name_is_reported_with_the_closest_one() {
    let found = only("@allow(float_equalty)\nlet x = 1\nprint(x)\n");
    assert_eq!(found.code, UNKNOWN_LINT.code);
    assert_eq!(found.message, "there is no lint called `float_equalty`");
    assert_eq!(
        found.help.as_deref(),
        Some("did you mean `float_equality`?")
    );
    let far = only("@allow(zzz)\nlet x = 1\nprint(x)\n");
    assert_eq!(
        far.help.as_deref(),
        Some("`cpl lint --list` prints every lint")
    );
}

#[test]
fn cp_toml_sets_levels() {
    let src = "var total = 0\nlet x: F64 = read()\nprint(total, x == 1.0)\n";
    assert_eq!(codes(src), ["L0201", "L0301"]);
    let toml = "[project]\nname = \"x\"\n\n[lints]\nvar_never_reassigned = \"allow\"  # noisy here\nfloat_equality = \"deny\"\n\n[judge]\nfloat_equality = \"allow\"\n";
    let (config, errors) = Config::from_toml(toml);
    assert_eq!(errors, []);
    let found = lint(src, &config);
    assert_eq!(found.len(), 1);
    assert_eq!(
        (found[0].code.0, found[0].severity),
        ("L0301", Severity::Error)
    );
}

#[test]
fn cp_toml_mistakes_are_reported_by_line() {
    let toml = "[lints]\nfloat_equalty = \"deny\"\nunread_field = \"loud\"\nmemory_mb = lots\nshadows_top_level\nunread_field = \"allow\"\n";
    let (config, errors) = Config::from_toml(toml);
    let lines: Vec<usize> = errors.iter().map(|e| e.line).collect();
    assert_eq!(lines, [2, 3, 4, 5]);
    assert_eq!(
        errors[0].to_string(),
        "line 2: there is no lint called `float_equalty`; did you mean `float_equality`?"
    );
    // The good line still applies.
    assert_eq!(config.level(&UNREAD_FIELD), Level::Allow);
    assert_eq!(config.memory_mb, config::DEFAULT_MEMORY_MB);
}

#[test]
fn overlapping_fixes_wait_for_the_next_round() {
    let a = Diagnostic::warning(Code("L0000"), Span::new(0, 3), "a").with_fix(
        "a",
        Span::new(0, 3),
        "let",
    );
    let b = Diagnostic::warning(Code("L0000"), Span::new(2, 5), "b").with_fix(
        "b",
        Span::new(2, 5),
        "zzz",
    );
    let c = Diagnostic::warning(Code("L0000"), Span::new(6, 7), "c").with_fix(
        "c",
        Span::new(6, 7),
        "yy",
    );
    assert_eq!(
        apply_fixes("var x=1\n", &[a, b, c]),
        ("let x=yy\n".to_string(), 2)
    );
}

/// Every program under `proto/`: the linter reads it without panicking and
/// every span it reports lies inside the file, on the text it names.
#[test]
fn lints_the_whole_corpus() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../proto");
    let mut seen = 0;
    for dir in ["ports", "tests/ok", "tests/lend", "tests/fn", "tests/bad"] {
        let Ok(entries) = std::fs::read_dir(root.join(dir)) else {
            continue;
        };
        for entry in entries.flatten() {
            if entry.path().extension().is_none_or(|e| e != "cp") {
                continue;
            }
            let src = std::fs::read_to_string(entry.path()).unwrap();
            seen += 1;
            for found in lint(&src, &Config::default()) {
                assert!(
                    found.span.end as usize <= src.len(),
                    "{}",
                    entry.path().display()
                );
                assert!(
                    !found.span.is_empty(),
                    "{}: {found:?}",
                    entry.path().display()
                );
                if found.code == VAR_NEVER_REASSIGNED.code {
                    assert_eq!(found.span.text(&src), "var", "{}", entry.path().display());
                }
            }
        }
    }
    assert!(seen >= 40, "found only {seen} corpus files");
}

mod cli {
    use crate::cli::run;
    use std::path::PathBuf;

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("cpl-lint-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn lint(args: &[&str]) -> (u8, String, String) {
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
    fn warnings_do_not_fail_and_deny_does() {
        let dir = scratch("levels");
        let file = dir.join("a.cp");
        std::fs::write(&file, "var total = 0\nprint(total)\n").unwrap();
        let (code, out, err) = lint(&[file.to_str().unwrap()]);
        assert_eq!(code, 0, "{err}");
        assert!(
            out.starts_with("warning[L0201]: `total` is declared `var` but never changed"),
            "{out}"
        );
        assert!(out.contains("a.cp:1:1"), "{out}");
        assert!(err.contains("1 file checked, 1 warning, 0 errors"), "{err}");

        std::fs::write(
            dir.join("cp.toml"),
            "[lints]\nvar_never_reassigned = \"deny\"\n",
        )
        .unwrap();
        let (code, out, err) = lint(&[dir.to_str().unwrap()]);
        assert_eq!(code, 1);
        assert!(out.starts_with("error[L0201]"), "{out}");
        assert!(err.contains("0 warnings, 1 error"), "{err}");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn fix_rewrites_the_file_and_reports_what_is_left() {
        let dir = scratch("fix");
        let file = dir.join("a.cp");
        std::fs::write(
            &file,
            "var a = 1\nvar b = 2\nlet x: F64 = read()\nprint(a, b, x == 1.0)\n",
        )
        .unwrap();
        let (code, out, err) = lint(&["--fix", file.to_str().unwrap()]);
        assert_eq!(code, 0);
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "let a = 1\nlet b = 2\nlet x: F64 = read()\nprint(a, b, x == 1.0)\n"
        );
        assert!(out.contains("L0301") && !out.contains("L0201"), "{out}");
        assert!(
            err.contains("1 warning, 0 errors, 2 fixes applied"),
            "{err}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_file_that_does_not_parse_is_an_error_not_a_pass() {
        let dir = scratch("broken");
        let file = dir.join("a.cp");
        std::fs::write(&file, "let x = (1\n").unwrap();
        let (code, out, err) = lint(&[file.to_str().unwrap()]);
        assert_eq!((code, out.as_str()), (1, ""));
        assert!(
            err.contains("has syntax errors and was not linted"),
            "{err}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn a_bad_cp_toml_is_reported_once() {
        let dir = scratch("toml");
        std::fs::write(dir.join("cp.toml"), "[lints]\nnope = \"deny\"\n").unwrap();
        std::fs::write(dir.join("a.cp"), "print(1)\n").unwrap();
        std::fs::write(dir.join("b.cp"), "print(2)\n").unwrap();
        let (code, _, err) = lint(&[dir.to_str().unwrap()]);
        assert_eq!(code, 0);
        assert_eq!(
            err.matches("there is no lint called `nope`").count(),
            1,
            "{err}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn list_help_and_bad_options() {
        let (code, out, _) = lint(&["--list"]);
        assert_eq!(code, 0);
        assert_eq!(out.lines().count(), crate::LINTS.len());
        assert!(out.contains("L0412  domain_keyed_map"), "{out}");
        let (code, out, _) = lint(&["--help"]);
        assert_eq!(code, 0);
        assert!(out.starts_with("usage: cpl lint"), "{out}");
        let (code, _, err) = lint(&["--loud"]);
        assert_eq!(code, 2);
        assert!(err.contains("unknown option `--loud`"), "{err}");
        let (code, _, err) = lint(&["/no/such/file.cp"]);
        assert_eq!(code, 1);
        assert!(err.contains("can't read /no/such/file.cp"), "{err}");
    }
}

#[test]
fn an_operator_passed_as_a_fn_hides_no_name() {
    // `+` stands for `(a, b) => a + b`; the `a` of that is not the writer's.
    let src = "let a = [1, 2]\nlet it = 3\nfn total(f: (Int, Int) -> Int, xs: List<Int>) -> Int:\n    var t = 0\n    for x in xs: t = f(t, x)\n    return t\nfn go():\n    print(total(+, a), it)\n    print(sorted(a, by: .abs))\ngo()\n";
    assert_eq!(codes(src), Vec::<&str>::new());
}

#[test]
fn a_top_level_loop_hides_no_name_written_below_it() {
    // Top-level code runs in order, so `i` of the loop is gone before
    // `var i` exists. A fn sees the top level whole, so there it does hide.
    let src = "for i in 0..3:\n    print(i)\nvar i = 5\ni += 1\nprint(i)\n";
    assert_eq!(codes(src), Vec::<&str>::new());
    let src = "fn go():\n    for i in 0..3:\n        print(i)\nvar i = 5\ni += 1\ngo()\nprint(i)\n";
    assert_eq!(codes(src), vec!["L0203"]);
    let src = "var i = 5\ni += 1\nfor i in 0..3:\n    print(i)\n";
    assert_eq!(codes(src), vec!["L0203"]);
}

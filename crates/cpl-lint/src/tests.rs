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
    let src = "let n: 1..=2e5 = read()\ndomain V = 1..=n\nlet a: [V]Int = read()\nprint(max(a[v] for v in V))\n";
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
    let src =
        "let n: 1..=9 = read()\nfn f(k: Int) -> Int:\n    let n = k + 1\n    n\nprint(f(n))\n";
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
    let dense = format!(
        "{head}var seen: Set<V> = Set<V>()\nseen.insert(V.first)\nprint(V.first in seen)\n"
    );
    assert_eq!(codes(&dense), ["L0412"]);
    // A set per `V` holds few of them; `[V][V]Bool` would hold all.
    let rows = format!("{head}var adj: [V]Set<V> = Set<V>()\nadj[V.first].insert(V.last)\nprint(adj[V.first].len)\n");
    assert_eq!(codes(&rows), Vec::<&str>::new());
    let listed = format!("{head}let groups: List<Set<V>> = []\nprint(groups.len)\n");
    assert_eq!(codes(&listed), Vec::<&str>::new());
    // Asked for its smallest or largest, which an array cannot answer.
    for ask in [
        "free.first",
        "free.last",
        "free.pop_first()",
        "free.pop_last()",
        "free.first_ge(V.first)",
    ] {
        let src =
            format!("{head}var free: Set<V> = Set<V>()\nfree.insert(V.first)\nprint({ask})\n");
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
    // So does one reading it by its bare name; a bare `id = ..` is a write.
    assert_eq!(codes(&method.replace("self.id", "id")), Vec::<&str>::new());
    let set = "struct Edge:\n    id: Int\n    fn clear(mut self):\n        id = 0\nvar e: Edge = read()\ne.clear()\n";
    assert_eq!(only(set).message, "field `id` of `Edge` is never read");
    // A parameter or a local named like the field is not the field.
    for body in ["    fn pick(self, id: Int) -> Int = id + 1\n", "    fn pick(self, k: Int) -> Int:\n        let id = k * 2\n        id\n", "    fn pick(self, k: Int) -> Int:\n        var s = k\n        for id in 0..k: s += id\n        s\n"] {
        let src = format!("struct Edge:\n    id: Int\n{body}var e: Edge = read()\nprint(e.pick(3))\n");
        assert_eq!(only(&src).message, "field `id` of `Edge` is never read", "{body}");
    }
    // Once the local is out of reach the bare name is the field again.
    let after = "struct Edge:\n    id: Int\n    fn pick(self, k: Int) -> Int:\n        for id in 0..k: print(id)\n        id\nvar e: Edge = read()\nprint(e.pick(3))\n";
    assert_eq!(codes(after), Vec::<&str>::new());
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
    fn taste_reports_a_line_per_file_and_the_total() {
        let dir = scratch("taste");
        // proto/ports/scout_batch_37/noip18p4.cp's edge list, beside a clean file.
        std::fs::write(
            dir.join("a.cp"),
            "var adj: [0..3]List<Int> = []\nlet (u, v) = (0, 2)\nadj[u].push(v)\nadj[v].push(u)\nvar total = 0\nprint(adj[0].len)\n",
        )
        .unwrap();
        std::fs::write(dir.join("b.cp"), "print([1, 2].sum())\n").unwrap();
        let (code, out, err) = lint(&["--taste", dir.to_str().unwrap()]);
        assert_eq!(code, 0, "{err}");
        let a = dir.join("a.cp");
        // `var total` is L0201, not a taste lint: it is left out.
        assert_eq!(
            out,
            format!(
                "   1  {}  L0503:1\n   1  total over 1 file of 2  L0503:1\n",
                a.display()
            )
        );
        assert_eq!(err, "2 files checked, 1 warning, 0 errors\n");
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
    fn fix_codemods_removed_where_filters_and_multitest_wrap_lines() {
        let dir = scratch("fix-removed");
        let file = dir.join("a.cp");
        std::fs::write(
            &file,
            "--- with multitest() ---\nlet xs = [1, 2, 3]\nlet ys = [x for x in xs where x > 1]\nfor x in ys where x < 3:\n    print(x)\n",
        )
        .unwrap();
        let (code, _out, err) = lint(&["--fix", file.to_str().unwrap()]);
        assert_eq!(code, 0, "{err}");
        assert_eq!(
            std::fs::read_to_string(&file).unwrap(),
            "@!multitest\nlet xs = [1, 2, 3]\nlet ys = [x for x in xs if x > 1]\nfor x in ys if x < 3:\n    print(x)\n"
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
    let src = "let a = [1, 2]\nlet it = 3\nfn total(f: (Int, Int) -> Int, xs: List<Int>) -> Int:\n    var t = 0\n    for x in xs: t = f(t, x)\n    return t\nfn go():\n    print(total(+, a), it)\n    print(a.sorted(by: .abs))\ngo()\n";
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

/// Below `@!multitest` every name belongs to one test, and a fn above the
/// line can't name it: a local of that fn hides nothing.
#[test]
fn a_local_above_multitest_hides_nothing_below_it() {
    let helper = "fn helper(k: Int) -> Int:\n    let n = k * 2\n    var total = 0\n    for i in 0..n:\n        total += i\n        total *= 2\n    total\n";
    let tests = "let n: Int = read()\nlet total = helper(n)\nprint(total)\n";
    assert_eq!(
        codes(&format!("{helper}@!multitest\n{tests}")),
        Vec::<&str>::new()
    );
    // Without the line, and for a fn below it, the names are in reach.
    assert_eq!(codes(&format!("{helper}{tests}")), ["L0203", "L0203"]);
    assert_eq!(
        codes(&format!("@!multitest\n{helper}{tests}")),
        ["L0203", "L0203"]
    );
    // What is above the line is still hidden by a local of a fn above it.
    assert_eq!(
        codes(&format!(
            "let n = 3\n{helper}@!multitest\nlet total = helper(n)\nprint(total)\n"
        )),
        ["L0203"]
    );
}

#[test]
fn a_statement_s_own_names_are_not_hidden_on_its_right_side() {
    let own = "let n = 3\nlet (i, j) = first((i, j) for i in 0..n for j in 0..n if i + j == 3)!\nprint(i, j)\n";
    assert!(codes(own).is_empty());
    let later =
        "let n = 3\nlet i = 1\nlet best = first(i for i in 0..n if i > 1)!\nprint(i, best)\n";
    assert_eq!(codes(later), ["L0203"]);
}

/// Applies every fix until none is left; the result must lint clean of `code`.
fn fixed(src: &str) -> String {
    let mut src = src.to_string();
    loop {
        let found = lint(&src, &Config::default());
        let (text, applied) = apply_fixes(&src, &found);
        if applied == 0 {
            return src;
        }
        src = text;
    }
}

fn taste_codes(src: &str) -> Vec<&'static str> {
    codes(src)
        .into_iter()
        .filter(|c| c.starts_with("L05"))
        .collect()
}

#[test]
fn search_loop_becomes_first_from_shoi16p4() {
    // proto/ports/scout_batch_29/shoi16p4.cp: the pivot of a Gaussian elimination.
    let src = "fn pivot_of(a: List<Int>, i: Int) -> ?Int:\n    var pivot: ?Int = none\n    for r in i..a.len:\n        if a[r] != 0:\n            pivot = r\n            break\n    pivot\nprint(pivot_of([0, 3], 0))\n";
    assert_eq!(taste_codes(src), ["L0501"]);
    assert_eq!(
        fixed(src),
        "fn pivot_of(a: List<Int>, i: Int) -> ?Int:\n    let pivot: ?Int = first(r for r in i..a.len if a[r] != 0)\n    pivot\nprint(pivot_of([0, 3], 0))\n"
    );
    // An open range is bracketed, so its `if` is not read as its end.
    let open = "fn f(a: List<Int>):\n    var at: ?Int = none\n    for r in 0..:\n        if a[r] != 0:\n            at = r\n            break\n    print(at)\n";
    assert!(fixed(open).contains("first(r for r in (0..) if a[r] != 0)"));
}

#[test]
fn search_loop_with_a_flag_becomes_any_or_all_from_noi09p1() {
    // proto/ports/scout_batch_15/noi09p1.cp: `var moved = false` over a neighbour list.
    let src = "let adj = [1, 2]\nvar moved = false\nfor a in adj:\n    if a > 1:\n        moved = true\n        break\nprint(moved)\n";
    assert_eq!(taste_codes(src), ["L0501"]);
    assert!(fixed(src).contains("let moved = any(a > 1 for a in adj)"));
    let all = "let grp = [1, 2]\nvar ok = true\nfor m in grp:\n    if m < 0:\n        ok = false\n        break\nprint(ok)\n";
    assert!(fixed(all).contains("let ok = all(not (m < 0) for m in grp)"));
    // A loop that does more than search is flagged without a rewrite.
    let more = "let grp = [1, 2]\nvar ok = true\nfor m in grp:\n    print(m)\n    if m < 0:\n        ok = false\n        break\nprint(ok)\n";
    let found = only(more);
    assert_eq!((found.code.0, found.fixes.len()), ("L0501", 0));
}

#[test]
fn discarded_result_from_noi23p4() {
    // proto/ports/scout_batch_52/today/noi23p4.cp: `let _ = save_sub(u * 2)`.
    let src =
        "fn save_sub(u: Int) -> Int:\n    print(u)\n    u\nlet _ = save_sub(2)\nlet _ = print(1)\n";
    let found = only(src);
    assert_eq!(found.code, DISCARDED_RESULT.code);
    assert_eq!(
        found.message,
        "`let _ =` throws away what `save_sub` returns"
    );
}

#[test]
fn symmetric_writes_from_noip18p4_and_noi07p1() {
    // proto/ports/scout_batch_37/noip18p4.cp and scout_batch_23/noi07p1.cp.
    let pushes = "var adj: [0..3]List<Int> = []\nlet (u, v) = (0, 2)\nadj[u].push(v)\nadj[v].push(u)\nprint(adj[0].len)\n";
    assert_eq!(taste_codes(pushes), ["L0503"]);
    let cells = "var ways: [0..3, 0..3]Int = 0\nlet (u, v) = (0, 2)\nways[u, v] = 1\nways[v, u] = 1\nprint(ways[0, 2])\n";
    assert_eq!(taste_codes(cells), ["L0503"]);
    let weighted = "var adj: [0..3]List<(Int, Int)> = []\nlet (u, v) = (0, 2)\nadj[u].push((v, 5))\nadj[v].push((u, 5))\nprint(adj[0].len)\n";
    assert_eq!(taste_codes(weighted), ["L0503"]);
    // Different facts, or a self-loop, are not one fact twice.
    let other = "var adj: [0..3]List<(Int, Int)> = []\nlet (u, v) = (0, 2)\nadj[u].push((v, 5))\nadj[v].push((u, 6))\nadj[u].push(u)\nadj[u].push(u)\nprint(adj[0].len)\n";
    assert_eq!(taste_codes(other), Vec::<&str>::new());
}

#[test]
fn loop_reductions_from_cco23p4_and_powers() {
    // proto/ports/scout_batch_36/cco23p4.cp: occurrences of a pattern.
    let count = "fn occ(w: Str, pat: Str) -> Int:\n    var res = 0\n    for i in 0..=w.len - pat.len:\n        if w[i..i + pat.len] == pat: res += 1\n    res\nprint(occ(\"aaa\", \"aa\"))\n";
    assert_eq!(taste_codes(count), ["L0504"]);
    assert_eq!(
        fixed(count),
        "fn occ(w: Str, pat: Str) -> Int:\n    let res = count(w[i..i + pat.len] == pat for i in 0..=w.len - pat.len)\n    res\nprint(occ(\"aaa\", \"aa\"))\n"
    );
    // examples/local-lib/math/powers.cp: the declared type stays.
    let sum = "const P = 7\nlet pw: List<Mod<P>> = [1, 2]\nvar acc: Mod<P> = 0\nfor i in 0..2: acc += pw[i]\nprint(acc)\n";
    assert!(fixed(sum).contains("let acc: Mod<P> = sum(pw[i] for i in 0..2)\nprint(acc)"));
    // `max=` is named but not rewritten: `max` of nothing has no value.
    let best = "let xs = [3, 1]\nvar best = 1_000_000_000\nfor x in xs: best min= x\nprint(best)\n";
    let found = only(best);
    assert_eq!((found.code.0, found.fixes.len()), ("L0504", 0));
    // Strings, a `?`, a name of the file's own, and self-reference are left alone.
    let skip = "let xs = [\"a\"]\nvar s = \"\"\nfor x in xs: s += x\nvar t = 0\nfor x in [1]: t += t\nprint(s, t)\n";
    assert_eq!(taste_codes(skip), Vec::<&str>::new());
    let tried = "fn g(k: Int) -> ?Int = k\nfn f() -> ?Int:\n    var s = 0\n    for k in 0..4: s += g(k)?\n    s\nprint(f())\n";
    assert_eq!(lint(tried, &Config::default())[0].fixes.len(), 0);
}

#[test]
fn index_only_loop_from_noi05p2() {
    // proto/ports/scout_batch_24/today/noi05p2.cp.
    let src = "let vals = [3, 8]\nfor i in 0..vals.len:\n    print(vals[i] * 2, vals[i])\n";
    assert_eq!(taste_codes(src), ["L0505"]);
    assert_eq!(
        fixed(src),
        "let vals = [3, 8]\nfor val in vals:\n    print(val * 2, val)\n"
    );
    // `i` used for itself, or the list written through it, needs the index.
    let needed = "var vals = [3, 8]\nfor i in 0..vals.len:\n    print(i, vals[i])\nfor i in vals.indices:\n    vals[i] += 1\nprint(vals)\n";
    assert_eq!(taste_codes(needed), Vec::<&str>::new());
}

#[test]
fn push_loop_becomes_a_comprehension_from_ioi16p6io_and_dmopc21c5p5() {
    // proto/ports/scout_batch_14/ioi16p6io.cp: segments normalised one push at a time.
    let src = "let raw = [(r: 2, c: 1)]\nvar norm: List<(Int, Int)> = []\nfor p in raw: norm.push((min(p.r, p.c), max(p.r, p.c)))\nnorm.push((0, 0))\nprint(norm)\n";
    assert_eq!(taste_codes(src), ["L0507"]);
    assert_eq!(
        fixed(src),
        "let raw = [(r: 2, c: 1)]\nvar norm: List<(Int, Int)> = [(min(p.r, p.c), max(p.r, p.c)) for p in raw]\nnorm.push((0, 0))\nprint(norm)\n"
    );
    // proto/ports/blind_critic_4/dmopc21c5p5.cp, with a filter and a head.
    let head = "fn f(n: Int) -> List<Int>:\n    var perm: List<Int> = [0]\n    for i in 1..=n:\n        if i % 2 == 1: perm.push(i)\n    perm\nprint(f(3))\n";
    assert!(fixed(head).contains("let perm: List<Int> = [0] + [i for i in 1..=n if i % 2 == 1]"));
    // A push that reads the list is a recurrence, not a comprehension.
    let rec = "var fib = [1, 1]\nfor i in 0..5: fib.push(fib[i] + fib[i + 1])\nprint(fib)\n";
    assert_eq!(taste_codes(rec), Vec::<&str>::new());
}

#[test]
fn none_check_unwrap_from_cco16p3() {
    // proto/ports/scout_batch_15/cco16p3.cp: `if p != none and v == p!: continue`.
    let src = "fn f(p: ?Int, v: Int) -> Bool = p != none and v == p!\nprint(f(none, 1))\n";
    let found = only(src);
    assert_eq!(found.code, NONE_CHECK_UNWRAP.code);
    assert_eq!(
        found.message,
        "`p != none` and then `p!`: the check is said twice"
    );
    let other = "fn f(p: ?Int, q: ?Int) -> Bool = p != none and q! == 1\nprint(f(none, 1))\n";
    assert_eq!(taste_codes(other), Vec::<&str>::new());
}

#[test]
fn repeated_literal_from_noip19p4() {
    // proto/ports/scout_batch_29/noip19p4.cp: the modulus in a type and again in a `%`.
    let src = "type Z = Mod<998244353>\nlet a = [3, 4]\nlet total = a.sum() % 998244353\nprint(total, Z(total))\n";
    let found = only(src);
    assert_eq!(found.code, REPEATED_LITERAL.code);
    assert_eq!(
        found.message,
        "`998244353` is written on 2 lines; it is first on line 1"
    );
    // proto/ports/scout_batch_39/today/aac4p5.cp: equal statement bounds of
    // different inputs are separate facts; `-1e9..=1e9` is one; small numbers repeat.
    let bounds = "let m: 0..=500_000 = read()\nlet q: 0..=500_000 = read()\nlet x: -1e9..=1e9 = read()\nprint(m + 1000, q - 1000, x + 1000)\n";
    assert_eq!(taste_codes(bounds), Vec::<&str>::new());
}

#[test]
fn turbofish_type_args_offers_plain_generic_fix() {
    let src = "let x = read::<Int>()\nprint(x)\n";
    let found = only(src);
    assert_eq!(found.code, TURBOFISH_TYPE_ARGS.code);
    let (fixed, applied) = apply_fixes(src, &[found]);
    assert_eq!(applied, 1);
    assert_eq!(fixed, "let x = read<Int>()\nprint(x)\n");
    assert!(lint(&fixed, &Config::default()).is_empty());
}

#[test]
fn free_fold_becomes_a_method() {
    // proto/ports: `sum(a)`, `count(xs, p)`, `sort(mut xs, by:)` beside `xs.all(p)`.
    let src = "let xs = [3, 1, 4]\nvar ys = [2, 7]\nsort(mut ys, by: y => -y)\nprint(sum(xs), count(xs, x => x > 2), max(xs[1..]), sorted(xs + ys))\n";
    assert_eq!(taste_codes(src), ["L0509"; 5]);
    assert_eq!(
        fixed(src),
        "let xs = [3, 1, 4]\nvar ys = [2, 7]\nys.sort(by: y => -y)\nprint(xs.sum(), xs.count(x => x > 2), xs[1..].max(), (xs + ys).sorted())\n"
    );
    let find = "let xs = [3, 1, 4]\nprint(xs.find(x => x > 3), xs.find(4))\n";
    assert_eq!(
        fixed(find),
        "let xs = [3, 1, 4]\nprint(xs.first(x => x > 3), xs.find(4))\n"
    );
    // Generators, two values, a domain, a list of values and a fn of the file stay free.
    let math = "domain V = 0..3\nfn sum(a: Int) -> Int = a\nlet xs = [3, 1]\nprint(count(x > 1 for x in xs), min(xs[0], xs[1]), max([xs[0], 2]), sorted(V, by: v => -v.ord), sum(2))\n";
    assert_eq!(taste_codes(math), Vec::<&str>::new());
}

#[test]
fn by_twin_becomes_by_option() {
    let src = "let xs = [\"bbb\", \"a\", \"cc\"]\nprint(min_by(xs, s => s.len), max_by(0..3, i => -i), xs.runs_by(s => s.len), xs.first_ge_by(2, s => s.len))\n";
    assert_eq!(taste_codes(src), ["L0510"; 4]);
    assert_eq!(
        fixed(src),
        "let xs = [\"bbb\", \"a\", \"cc\"]\nprint(xs.min(by: s => s.len), (0..3).max(by: i => -i), xs.runs(by: s => s.len), xs.first_ge(2, by: s => s.len))\n"
    );
    // Domain and comprehension free calls stay free with `by:`.
    let dom = "domain V = 0..3\nprint(max_by(V, v => v.ord), xs.run_ranges_by(by: x => x))\n";
    assert_eq!(
        fixed(dom),
        "domain V = 0..3\nprint(max(V, by: v => v.ord), xs.run_ranges(by: x => x))\n"
    );
    // `chunk_by` takes a binary predicate, not a `by:` key, so it is untouched.
    let chunk = "let xs = [1, 1, 2]\nprint(xs.chunk_by((a, b) => a == b))\n";
    assert_eq!(taste_codes(chunk), Vec::<&str>::new());
}
#[test]
fn tree_order_becomes_preorder_array() {
    let src = "for v in t.preorder.rev: print(v)\nfor v in t.postorder: print(v)\nlet top = t.preorder[0]\n";
    assert_eq!(taste_codes(src), ["L0511"; 3]);
    assert_eq!(
        fixed(src),
        "for v in t.at.rev: print(v)\nfor v in t.at.rev: print(v)\nlet top = t.at[0]\n"
    );
}
#[test]
fn domain_spelling_and_group_by_codemods() {
    let src = "\
domain V = 1..=3
let w: [V]Int = [3, 1, 2]
let a = V.sorted(by: w)
let b = V.first(where: v => w[v] > 1)
let c = V.group_by(v => Int(v) % 2)
";
    assert_eq!(taste_codes(src), ["L0512", "L0512", "L0510"]);
    assert_eq!(
        fixed(src),
        "\
domain V = 1..=3
let w: [V]Int = [3, 1, 2]
let a = V.sort(by: w)
let b = V.first(v => w[v] > 1)
let c = V.group(by: v => Int(v) % 2)
"
    );
}

#[test]
fn sorted_range_method_codemod() {
    let src = "\
var s = SortedBag<Int>()
let a = s.first_ge(3)
let b = s.first_gt(3)
let c = s.last_le(5)
let d = s.last_lt(5)
let e = s.count_range(1, 5)
";
    assert_eq!(
        taste_codes(src),
        ["L0515", "L0515", "L0515", "L0515", "L0515"]
    );
    assert_eq!(
        fixed(src),
        "\
var s = SortedBag<Int>()
let a = s.range(3..).first
let b = s.range(3 + 1..).first
let c = s.range(..=5).last
let d = s.range(..5).last
let e = s.range(1..5).len
"
    );
}

#[test]
fn prefix_twin_becomes_prefix() {
    let src = "\
use std.prefix.{prefix1, Prefix2, Diff1, Diff2}
let xs = [3, 1, 4]
let a = prefix_n(xs)
let b = prefix1([0..3] i => i)
let c = prefix2([0..2, 0..2] (r, c) => r + c)
let d1 = Diff1<V, Int>()
let d2 = Diff2<R, C, Int>()
fn f(p: Prefix1<V, Int>, q: Prefix2<R, C, Int>, r: PrefixN<Int>) -> Int = 0
";
    assert_eq!(taste_codes(src), ["L0513"; 9]);
    assert_eq!(
        fixed(src),
        "\
let xs = [3, 1, 4]
let a = prefix(xs)
let b = prefix([0..3] i => i)
let c = prefix([0..2, 0..2] (r, c) => r + c)
let d1 = Diff<V, Int>()
let d2 = Diff<R, C, Int>()
fn f(p: Prefix<V, Int>, q: Prefix<R, C, Int>, r: Prefix<Int>) -> Int = 0
"
    );
}

#[test]
fn hand_rolled_range_tree_flags_bit_and_segtree_without_false_positives() {
    let fenwick_add = "\
var bit: [0..=8]Int = 0
var p = 3
while p <= 8:
    bit[p] += 1
    p += p & -p
";
    assert_eq!(taste_codes(fenwick_add), ["L0514"]);

    let fenwick_zero_idx = "\
var bit: [0..8]Int = 0
var i = 2
while i < 8:
    bit[i] += 1
    i = i | (i + 1)
";
    assert_eq!(taste_codes(fenwick_zero_idx), ["L0514"]);

    let iter_seg_pull = "\
var t: [0..16]Int = 0
var p = 4
while p > 0:
    t[p] = t[2 * p] + t[2 * p + 1]
    p >>= 1
";
    assert_eq!(taste_codes(iter_seg_pull), ["L0514"]);

    let iter_seg_fold = "\
var t: [0..16]Int = 0
var l = 8
var r = 13
var acc = 0
while l < r:
    if (l & 1) == 1:
        acc += t[l]
        l += 1
    if (r & 1) == 1:
        r -= 1
        acc += t[r]
    l >>= 1
    r >>= 1
";
    assert_eq!(taste_codes(iter_seg_fold), ["L0514"]);

    let rec_seg = "\
var seg: [0..32]Int = 0
fn upd(node: Int, lo: Int, hi: Int, ql: Int, qr: Int):
    if qr <= lo or hi <= ql: return
    if ql <= lo and hi <= qr:
        seg[node] += 1
        return
    let mid = (lo + hi) / 2
    upd(2 * node, lo, mid, ql, qr)
    upd(2 * node + 1, mid, hi, ql, qr)
";
    assert_eq!(taste_codes(rec_seg), ["L0514"]);

    // Negative cases: ordinary arithmetic, bit-counting, and polynomial divide-and-conquer.
    let clean = "\
for k in 0..5:
    print(2 * k + 1)
var mask = 13
var bits = 0
while mask > 0:
    bits += 1
    mask -= mask & -mask
fn poly(node: Int, lo: Int, hi: Int):
    if lo + 1 == hi: return
    let mid = (lo + hi) / 2
    poly(2 * node, lo, mid)
    poly(2 * node + 1, mid, hi)
";
    assert_eq!(taste_codes(clean), Vec::<&str>::new());
}

#[test]
fn hand_rolled_string_algo_flags_kmp_z_hash_and_trie_without_false_positives() {
    let kmp = "\
let s = \"ababa\"
var pi = List.filled(s.len, 0)
for i in 1..s.len:
    var j = pi[i - 1]
    while j > 0 and s[i] != s[j]:
        j = pi[j - 1]
    if s[i] == s[j]:
        j += 1
    pi[i] = j
";
    assert_eq!(taste_codes(kmp), ["L0516"]);

    let z_fn = "\
let s = \"ababa\"
let n = s.len
var z = List.filled(n, 0)
var (l, r) = (0, 0)
for i in 1..n:
    if i < r: z[i] = min(r - i, z[i - l])
    while i + z[i] < n and s[z[i]] == s[i + z[i]]:
        z[i] += 1
    if i + z[i] > r:
        (l, r) = (i, i + z[i])
";
    assert_eq!(taste_codes(z_fn), ["L0516"]);

    let poly_hash = "\
let s = \"ababa\"
let n = s.len
const B = 131
const M = 1_000_000_007
var h = List.filled(n + 1, 0)
var pw = List.filled(n + 1, 1)
for i in 0..n:
    h[i + 1] = (h[i] * B + Int(s[i])) % M
    pw[i + 1] = (pw[i] * B) % M
";
    assert_eq!(taste_codes(poly_hash), ["L0516"]);

    let trie_loop = "\
struct Node:
    kid: [0..2]?Int
var nodes: List<Node> = [Node([0..2] _ => none)]
var o = 0
var rest = 12
while rest > 1:
    nodes[o].kid[rest % 2] ??= nodes.push_id(Node([0..2] _ => none))
    o = nodes[o].kid[rest % 2]!
    rest /= 2
";
    assert_eq!(taste_codes(trie_loop), ["L0516"]);

    // Negative cases: decimal parsing, tree DP child walk, and using std.strings.prefix_function.
    let clean = "\
let s = \"12345\"
var val = 0
for ch in s:
    val = val * 10 + Int(ch) - Int('0')
let pi = prefix_function(s)
var half = 0
for i in 1..s.len:
    while half > 0 and s[half] != s[i]: half = pi[half - 1]
    if s[half] == s[i]: half += 1
print(val, half)
";
    assert_eq!(taste_codes(clean), Vec::<&str>::new());
}

#[test]
fn loop_invariant_mod_div_hoists_inverse() {
    let basic = "type Z = Mod<998_244_353>\nlet d = Z(3)\nvar xs = [Z(1), Z(2)]\nfor i in 0..xs.len:\n    xs[i] = xs[i] / d\nprint(xs)\n";
    assert_eq!(codes(basic), ["L0415"]);
    let after = fixed(basic);
    assert_eq!(
        after,
        "type Z = Mod<998_244_353>\nlet d = Z(3)\nvar xs = [Z(1), Z(2)]\nlet inv_d = d.inv\nfor i in 0..xs.len:\n    xs[i] = xs[i] * inv_d\nprint(xs)\n"
    );
    assert_eq!(codes(&after), Vec::<&str>::new());

    let assign = "type Z = Mod<998_244_353>\nlet d = Z(5)\nvar xs = [Z(1), Z(2)]\nfor i in 0..xs.len:\n    xs[i] /= d\nprint(xs)\n";
    assert_eq!(codes(assign), ["L0415"]);
    assert_eq!(
        fixed(assign),
        "type Z = Mod<998_244_353>\nlet d = Z(5)\nvar xs = [Z(1), Z(2)]\nlet inv_d = d.inv\nfor i in 0..xs.len:\n    xs[i] *= inv_d\nprint(xs)\n"
    );

    // Int literal divisor on Mod LHS hoists Z(2).inv once for multiple sites.
    let int_lit = "type Z = Mod<998_244_353>\nvar a = [Z(1), Z(2)]\nvar b = [Z(3), Z(4)]\nfor i in 0..a.len:\n    a[i] = a[i] / 2\n    b[i] = b[i] / 2\nprint(a, b)\n";
    assert_eq!(codes(int_lit), ["L0415", "L0415"]);
    assert_eq!(
        fixed(int_lit),
        "type Z = Mod<998_244_353>\nvar a = [Z(1), Z(2)]\nvar b = [Z(3), Z(4)]\nlet inv_2 = Z(2).inv\nfor i in 0..a.len:\n    a[i] = a[i] * inv_2\n    b[i] = b[i] * inv_2\nprint(a, b)\n"
    );

    // Guarded divisor warns but does not auto-fix (d may be zero).
    let guarded = "type Z = Mod<998_244_353>\nlet d = Z(0)\nvar xs = [Z(1), Z(2)]\nfor i in 0..xs.len:\n    if d != Z(0):\n        xs[i] = xs[i] / d\nprint(xs)\n";
    assert_eq!(codes(guarded), ["L0415"]);
    assert_eq!(fixed(guarded), guarded);

    // Negative cases: loop-varying divisor (including Comb-in-scope small-int case),
    // mutated divisor, non-Mod division, trivial / 1, and outside a loop.
    let negative = "\
type Z = Mod<998_244_353>
let comb = Comb<Z>(10)
var d = Z(3)
var xs = [Z(1), Z(2)]
var ints = [4, 6]
let once = xs[0] / d
for i in 0..xs.len:
    xs[i] = xs[i] / (i + 1)
    xs[i] = xs[i] / 1
    xs[i] = xs[i] / d
    d += Z(1)
    ints[i] = ints[i] / 2
print(once, xs, ints, comb.fact(2))
";
    assert!(!codes(negative).contains(&"L0415"), "{:?}", codes(negative));
}

#[test]
fn pow_method_lint_warns_and_fixes_with_parentheses() {
    let src = "let a = 2\nlet n = 5\nprint(a.pow(3), a.pow(n - 1), (a + 1).pow(2), a.pow(n).abs)\n";
    assert_eq!(codes(src), vec!["L0205", "L0205", "L0205", "L0205"]);
    assert_eq!(
        fixed(src),
        "let a = 2\nlet n = 5\nprint(a ** 3, a ** (n - 1), (a + 1) ** 2, (a ** n).abs)\n"
    );
}

#[test]
fn getter_parens_lint_warns_and_drops_parens() {
    let src = "let xs = [1, 2, 3]\nprint(xs.len(), xs.is_empty(), xs.first(), xs.last(), xs.rev(), xs.items(), xs.first(x => x > 1))\n";
    assert_eq!(
        codes(src),
        vec!["L0206", "L0206", "L0206", "L0206", "L0206", "L0206"]
    );
    assert_eq!(
        fixed(src),
        "let xs = [1, 2, 3]\nprint(xs.len, xs.is_empty, xs.first, xs.last, xs.rev, xs.items, xs.first(x => x > 1))\n"
    );
}

#[test]
fn fix_rewrites_range_lo_hi_to_start_end_only_on_ranges() {
    let src = "\
struct Seg(lo: Int, hi: Int)
let s = Seg(1, 5)
let r = 2..=4
print(s.lo, s.hi, r.lo, r.hi)
";
    let (out, applied) = crate::fix(src, &Config::default());
    assert_eq!(applied, 2);
    assert_eq!(
        out,
        "\
struct Seg(lo: Int, hi: Int)
let s = Seg(1, 5)
let r = 2..=4
print(s.lo, s.hi, r.start, r.end)
"
    );
}

#[test]
fn hand_rolled_stdlib_algo_family_flags_dsu_sieve_binlift_and_segtree_pull() {
    let seg_build_pull = "\
var t: [0..16]Int = 0
for i in (1..8).rev:
    t[i] = t[2 * i] + t[2 * i + 1]
";
    assert_eq!(taste_codes(seg_build_pull), ["L0514"]);

    let dsu_halving = "\
domain V = 0..4
var up: [V]V = [V] v => v
fn find(v: V) -> V:
    var x = v
    while up[x] != x:
        up[x] = up[up[x]]
        x = up[x]
    x
";
    assert_eq!(taste_codes(dsu_halving), ["L0517"]);

    let dsu_recursive = "\
domain V = 0..4
var up: [V]V = [V] v => v
fn find(v: V) -> V:
    if up[v] == v: return v
    up[v] = find(up[v])
    up[v]
";
    assert_eq!(taste_codes(dsu_recursive), ["L0517"]);

    let dsu_two_pass = "\
domain V = 0..4
var up: [V]?V = none
fn find(v: V) -> V:
    var r = v
    while let p = up[r]: r = p
    var x = v
    while let p = up[x]:
        up[x] = r
        x = p
    r
";
    assert_eq!(taste_codes(dsu_two_pass), ["L0517"]);

    let eratosthenes = "\
let n = 30
var is_prime = List.filled(n + 1, true)
for i in 2..=n:
    if is_prime[i]:
        var j = i * i
        while j <= n:
            is_prime[j] = false
            j += i
";
    assert_eq!(taste_codes(eratosthenes), ["L0518"]);

    let linear_sieve = "\
let n = 30
var spf = List.filled(n + 1, 0)
var primes: List<Int> = []
for i in 2..=n:
    if spf[i] == 0:
        spf[i] = i
        primes.push(i)
    for p in primes:
        if p > spf[i] or i * p > n: break
        spf[i * p] = p
";
    assert_eq!(taste_codes(linear_sieve), ["L0518"]);

    let binlift_2d = "\
var up: [0..4, 0..8]Int = 0
for k in 1..4:
    for v in 0..8:
        up[k, v] = up[k - 1, up[k - 1, v]]
";
    assert_eq!(taste_codes(binlift_2d), ["L0519"]);

    let binlift_nested = "\
var up = [[0, 0, 1], [0, 0, 0]]
for k in 1..2:
    for v in 0..3:
        up[k][v] = up[k - 1][up[k - 1][v]]
";
    assert_eq!(taste_codes(binlift_nested), ["L0519"]);

    // Negative cases: ordinary tree parent chase without compression,
    // divisor-count multiple loop, ordinary 2D DP recurrence.
    let clean = "\
domain V = 0..4
let par: [V]V = [V] v => v
var x = V(3)
while par[x] != x: x = par[x]
let n = 20
var cnt = List.filled(n + 1, 0)
for i in 1..=n:
    var j = i
    while j <= n:
        cnt[j] += 1
        j += i
var dp: [0..4, 0..8]Int = 0
for k in 1..4:
    for v in 1..8:
        dp[k, v] = dp[k - 1, v] + dp[k - 1, v - 1]
";
    assert_eq!(taste_codes(clean), Vec::<&str>::new());
}

#[test]
fn is_in_cond_header_lint_warns_and_fixes_to_let() {
    let src = "\
let a: ?Int = some(3)
let b: ?Int = some(5)
let ok = a is v and v > 0
if a is x and x > 1:
    print(x)
elif b is y:
    print(y)
var cur: ?Int = some(2)
while cur is n and n > 0:
    print(n)
    cur = none
print(ok)
";
    assert_eq!(codes(src), vec!["L0207", "L0207", "L0207"]);
    assert_eq!(
        fixed(src),
        "\
let a: ?Int = some(3)
let b: ?Int = some(5)
let ok = a is v and v > 0
if let x = a, x > 1:
    print(x)
elif let y = b:
    print(y)
var cur: ?Int = some(2)
while let n = cur, n > 0:
    print(n)
    cur = none
print(ok)
"
    );
}

#[test]
fn hand_rolled_answer_search_becomes_bsearch_or_a_sorted_lookup() {
    // proto/ports/scout_batch_2/noip18p3.cp: a monotone predicate over an answer range.
    let first = "fn f(a: List<Int>, i: Int, k: Int) -> Int:\n    var (lo, hi) = (i + 1, a.len)  // the first chain at least k\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if a[mid] >= k - a[i]: hi = mid else: lo = mid + 1\n    lo\nprint(f([1, 2, 3], 0, 3))\n";
    assert_eq!(taste_codes(first), ["L0520"]);
    assert_eq!(
        fixed(first),
        "fn f(a: List<Int>, i: Int, k: Int) -> Int:\n    let lo = bsearch_first(i + 1..a.len, mid => a[mid] >= k - a[i]) ?? a.len  // the first chain at least k\n    lo\nprint(f([1, 2, 3], 0, 3))\n"
    );
    // The rounded-up midpoint keeps `lo`: the last true, with `lo` itself as the fallback.
    let last = "fn f(ok: Int -> Bool, n: Int) -> Int:\n    var lo = 0\n    var hi = n - 1\n    while lo < hi:\n        let mid = lo + (hi - lo + 1) / 2\n        if ok(mid): lo = mid\n        else: hi = mid - 1\n    lo\nprint(f(x => x < 3, 9))\n";
    assert_eq!(taste_codes(last), ["L0520"]);
    assert!(fixed(last).contains("let lo = bsearch_last(1..=n - 1, mid => ok(mid)) ?? 0\n"));
    // A sorted list compared at `mid` is the list's own lookup; the false branch is negated.
    let sorted = "fn f(xs: List<Int>, t: Int) -> Int:\n    var (lo, hi) = (0, xs.len)\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if xs[mid] < t: lo = mid + 1 else: hi = mid\n    lo\nprint(f([1, 4], 2))\n";
    assert!(fixed(sorted).contains("let lo = xs.first_ge(t) ?? xs.len\n"));
    let negated = "fn f(xs: List<Int>, t: Int) -> Int:\n    var (lo, hi) = (0, xs.len)\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if xs[mid] * 2 <= t: lo = mid + 1 else: hi = mid\n    lo\nprint(f([1, 4], 2))\n";
    assert!(fixed(negated).contains("bsearch_first(0..xs.len, mid => xs[mid] * 2 > t) ?? xs.len"));
    // `hi` read after the loop: a warning, no rewrite.
    let hi_read = "fn f(ok: Int -> Bool) -> Int:\n    var (lo, hi) = (0, 9)\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if ok(mid): hi = mid else: lo = mid + 1\n    hi\nprint(f(x => x > 3))\n";
    assert_eq!(taste_codes(hi_read), ["L0520"]);
    assert!(fixed(hi_read).contains("while lo < hi"));
    // Not flagged: a library's own loop under `unchecked:`, an interactive search,
    // and a loop that does more than halve.
    let clean = "\
fn first_big(a: List<Int>) -> Int:
    var (lo, hi) = (0, a.len)
    unchecked:
        while lo < hi:
            let mid = (lo + hi) / 2
            if a[mid] > 5: hi = mid else: lo = mid + 1
    lo
fn ask(x: Int) -> Bool:
    print(\"?\", x)
    flush()
    read() == 1
var (lo, hi) = (1, 100)
while lo < hi:
    let mid = (lo + hi) / 2
    if ask(mid): hi = mid else: lo = mid + 1
var steps = 0
var (a, b) = (0, 10)
while a < b:
    let mid = (a + b) / 2
    steps += 1
    if mid > 3: b = mid else: a = mid + 1
print(first_big([1, 9]), lo, steps)
";
    assert_eq!(taste_codes(clean), Vec::<&str>::new());
}
#[test]
fn a_prelude_import_is_dropped() {
    // proto/ports/ecnerwala/cf2124i.cp and fourteen more imported `Rooted`.
    let src = "use std.tree.Rooted\nuse std.mod.{Mod, Comb}\nuse std.rand.{Rng}\nuse std.graph.{Graph, Tree}\nlet x = 1\n";
    assert_eq!(codes(src), ["L0208"; 3]);
    assert_eq!(fixed(src), "use std.rand.{Rng}\nlet x = 1\n");
}

#[test]
fn a_prelude_name_from_another_module_or_renamed_is_kept() {
    let src = "use std.graph.{Rooted}\nuse std.tree.Rooted as R\nuse std.tree.*\nstruct Mod:\n    x: Int\nuse std.mod.{Mod, Comb}\nlet x = 1\n";
    assert_eq!(
        fixed(src).lines().filter(|l| l.starts_with("use")).count(),
        4,
        "{}",
        fixed(src)
    );
    assert!(fixed(src).contains("use std.mod.{Mod}\n"));
}

#[test]
fn a_library_keeps_its_prelude_imports() {
    let src = "use std.tree.{Rooted}\npub fn f(x: Int) -> Int = x\n";
    assert!(codes(src).is_empty());
}

#[test]
fn dropping_a_prelude_import_keeps_its_comment() {
    let src = "use std.geometry.{Pt, dot}  // WISH F1\nuse std.mod.{Mod}  // a note\nlet x = 1\n";
    assert_eq!(
        fixed(src),
        "use std.geometry.{dot}  // WISH F1\n// a note\nlet x = 1\n"
    );
}

#[test]
fn dropping_a_prelude_import_leaves_no_double_blank_line() {
    let src = "// a port\n\nuse std.tree.{Rooted}\n\nlet x = 1\n";
    assert_eq!(fixed(src), "// a port\n\nlet x = 1\n");
}

#[test]
fn dropping_leading_prelude_imports_leaves_no_leading_blank_line() {
    let src = "use std.segtree.{SegTree}\nuse std.algebra.{Monoid}\n\nlet x = 1\n";
    assert_eq!(fixed(src), "let x = 1\n");
}
#[test]
fn hand_rolled_prefix_sum_becomes_prefix() {
    // 1D List.filled + 0..a.len -> prefix(a) and p.prefix(...)
    let list_pref = "\
let a = [1, 2, 3, 4]
var p = List.filled(a.len + 1, 0)
for i in 0..a.len:
    p[i + 1] = p[i] + a[i]
print(p[3] - p[1])
";
    assert_eq!(taste_codes(list_pref), ["L0521"]);
    assert_eq!(
        fixed(list_pref),
        "let a = [1, 2, 3, 4]\nlet p = prefix(a)\nprint(p.prefix(3) - p.prefix(1))\n"
    );

    // 1D domain bounds (proto/ports/dpm.cp shape) -> prefix(ways) and pre.prefix(...)
    let dom_pref = "\
domain C = 0..=4
let ways: [C]Int = [1, 2, 3, 4, 5]
var pre: [C.bounds]Int = 0
for c in C: pre[c.r] = pre[c.l] + ways[c]
print(pre[C.last.r] - pre[C.first.l])
";
    assert_eq!(taste_codes(dom_pref), ["L0521"]);
    assert_eq!(
        fixed(dom_pref),
        "domain C = 0..=4\nlet ways: [C]Int = [1, 2, 3, 4, 5]\nlet pre = prefix(ways)\nprint(pre.prefix(C.last.r) - pre.prefix(C.first.l))\n"
    );

    // 1D [0..=n]Int with expression RHS -> prefix([... for i in 0..n])
    let range_pref = "\
let n = 4
let a = [1, 2, 3, 4]
var pre: [0..=n]Int = 0
for i in 0..n: pre[i + 1] = pre[i] + a[i] * 2
print(pre[n] - pre[1])
";
    assert_eq!(taste_codes(range_pref), ["L0521"]);
    assert_eq!(
        fixed(range_pref),
        "let n = 4\nlet a = [1, 2, 3, 4]\nlet pre = prefix([a[i] * 2 for i in 0..n])\nprint(pre.prefix(n) - pre.prefix(1))\n"
    );

    // 1D 1..=n and 2D inclusion-exclusion warn (no auto-fix).
    let one_based = "\
let n = 4
let a = [1, 2, 3, 4]
var pre = List.filled(n + 1, 0)
for i in 1..=n: pre[i] = pre[i - 1] + a[i - 1]
print(pre[n])
";
    assert_eq!(taste_codes(one_based), ["L0521"]);
    let grid_2d = "\
domain R = 0..3
domain C = 0..3
let g: [R, C]Int = 1
var p: [R.bounds, C.bounds]Int = 0
for r in R:
    for c in C:
        p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + g[r, c]
print(p[R.last.r, C.last.r])
";
    assert_eq!(taste_codes(grid_2d), ["L0521"]);

    // Mutated later: warns, but does not auto-fix.
    let mutated = "\
let a = [1, 2, 3]
var p = List.filled(a.len + 1, 0)
for i in 0..a.len:
    p[i + 1] = p[i] + a[i]
p[0] = 9
print(p[2])
";
    assert_eq!(taste_codes(mutated), ["L0521"]);
    assert!(fixed(mutated).contains("p[i + 1] = p[i] + a[i]"));

    // False-positive guards: Fibonacci, powers of two, DP recurrences, different source array.
    let clean = "\
let n = 6
var fib = List.filled(n + 1, 1)
for i in 2..=n: fib[i] = fib[i - 1] + fib[i - 2]
var pow2 = List.filled(n + 1, 1)
for i in 1..=n: pow2[i] = pow2[i - 1] + pow2[i - 1]
var ways = List.filled(n + 1, 1)
for i in 1..=n: ways[i] = ways[i - 1] + ways[i] * 2
var dp = List.filled(n + 1, 0)
for i in 1..=n: dp[i] = fib[i - 1] + 1
print(fib[n], pow2[n], ways[n], dp[n])
";
    assert_eq!(taste_codes(clean), Vec::<&str>::new());
}

#[test]
fn redundant_arg_name_shortens_to_punning() {
    let src = "\
fn f(moves: Int, cost: Int, other: Int) -> Int = moves + cost + other
let moves = 3
let cost = 4
let x = 5
let r = (moves: moves, val: x)
print(f(moves: moves, cost:, other: x) + r.moves)
";
    assert_eq!(codes(src), vec!["L0209", "L0209"]);
    assert_eq!(
        fixed(src),
        "\
fn f(moves: Int, cost: Int, other: Int) -> Int = moves + cost + other
let moves = 3
let cost = 4
let x = 5
let r = (moves:, val: x)
print(f(moves:, cost:, other: x) + r.moves)
"
    );
}

#[test]
fn copy_sort_writeback_rewrites_to_mut_slice_in_place() {
    // 2-statement form: `let sub = xs[l..r].sorted()` + index writeback.
    let two_stmt = "\
var xs = [5, 4, 3, 2, 1]
let sub = xs[1..4].sorted()
for i in 1..4: xs[i] = sub[i - 1]
print(xs)
";
    assert_eq!(taste_codes(two_stmt), ["L0522"]);
    assert_eq!(
        fixed(two_stmt),
        "var xs = [5, 4, 3, 2, 1]\nxs[1..4].sort()\nprint(xs)\n"
    );

    // 2-statement form with `by:`/`desc:` args and `.items` writeback.
    let with_args = "\
fn f(mut xs: List<Int>, l: Int, r: Int):
    let sub = xs[l..r].sorted(by: x => -x, desc: true)
    for (i, v) in sub.items: xs[l + i] = v
";
    assert_eq!(taste_codes(with_args), ["L0522"]);
    assert_eq!(
        fixed(with_args),
        "fn f(mut xs: List<Int>, l: Int, r: Int):\n    xs[l..r].sort(by: x => -x, desc: true)\n"
    );

    // 3-statement form: `.to_list()` + `.sort()` + `0..sub.len` writeback.
    let three_to_list = "\
var xs = [5, 4, 3, 2, 1]
var sub = xs[1..4].to_list()
sub.sort()
for i in 0..sub.len: xs[1 + i] = sub[i]
print(xs)
";
    assert_eq!(taste_codes(three_to_list), ["L0522"]);
    assert_eq!(
        fixed(three_to_list),
        "var xs = [5, 4, 3, 2, 1]\nxs[1..4].sort()\nprint(xs)\n"
    );

    // 3-statement form: list comprehension + `.reverse()` + range writeback.
    let three_comp = "\
fn f(mut xs: List<Int>, l: Int, r: Int):
    var sub = [xs[k] for k in l..r]
    sub.reverse()
    for i in l..r: xs[i] = sub[i - l]
";
    assert_eq!(taste_codes(three_comp), ["L0522"]);
    assert_eq!(
        fixed(three_comp),
        "fn f(mut xs: List<Int>, l: Int, r: Int):\n    xs[l..r].reverse()\n"
    );

    // `sub` read after writeback: warn, no auto-fix.
    let read_after = "\
var xs = [5, 4, 3, 2, 1]
let sub = xs[1..4].sorted()
for i in 1..4: xs[i] = sub[i - 1]
print(sub[0], xs)
";
    assert_eq!(taste_codes(read_after), ["L0522"]);
    assert!(fixed(read_after).contains("let sub = xs[1..4].sorted()"));

    // Strided comprehension is NOT a contiguous slice: no false positive.
    let strided = "\
var a = [4, 3, 2, 1]
var x = [a[i] for i in 0..4 by 2]
x.sort()
for (i, v) in x.items: a[2 * i] = v
print(a)
";
    assert_eq!(taste_codes(strided), Vec::<&str>::new());
}

#[test]
fn division_remainder_and_power_follow_the_interpreter() {
    let src = "let a: -7..=9 = read()\nlet b: 2..=4 = read()\nlet q = a / b\nlet r = a % b\nlet s = a ** 2\nlet c = a ** 3\nlet z = a / (b - 2)\n";
    let range = |name: &str| {
        let at = src.find(&format!("let {name} ")).unwrap() as u32 + 4;
        crate::proven_ranges(src)
            .into_iter()
            .find(|r| r.0 == at)
            .map(|r| (r.1, r.2))
    };
    // Euclidean, as `cpl run` computes it: -7 / 2 is -4 and -7 % 4 is 1.
    assert_eq!(range("q"), Some((-4, 4)));
    assert_eq!(range("r"), Some((0, 3)));
    assert_eq!(range("s"), Some((0, 81)));
    assert_eq!(range("c"), Some((-343, 729)));
    assert_eq!(range("z"), None, "the divisor can be 0");
}

fn range_of(src: &str, decl: &str) -> Option<(i128, i128)> {
    let at = src.find(decl).unwrap() as u32 + decl.find(' ').unwrap() as u32 + 1;
    crate::proven_ranges(src)
        .into_iter()
        .find(|r| r.0 == at)
        .map(|r| (r.1, r.2))
}

#[test]
fn a_var_is_in_the_hull_every_assignment_keeps() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        ("var best = 0\nfor i in 0..n:\n    best max= a * 2\nprint(best)\n", Some((0, 18))),
        ("var s = 5\ns = min(s, a)\ns = max(s, abs(a - 20))\nprint(s)\n", Some((0, 20))),
        ("var p = n\nif a > 3:\n    p = p % 7\nprint(p)\n", Some((0, 100))),
        ("var c = 0\nwhile c < n:\n    c += 1\nprint(c)\n", Some((0, 100))),
        ("var c = 0\nwhile c < n:\n    print(c)\n    c += 1\n    c += 1\nprint(c)\n", None),
        ("var c = 0\nfor i in 0..n:\n    c += 1\nprint(c)\n", None),
        ("var j = 0\nwhile j < n and a > 2:\n    j = j + 1\nprint(j)\n", Some((0, 100))),
        ("var t = 1\nfor i in 0..n:\n    t = t * 2\nprint(t)\n", None),
        ("var x = 0\nfor i in 0..n:\n    x = i\nprint(x)\n", None),
        ("var x = 0\nfn bump():\n    x = 50\nbump()\nprint(x)\n", Some((0, 50))),
        ("var x = 0\nvar y = 1\n(x, y) = (y, x + y)\nprint(x)\n", None),
        ("var x = 0\nlet x = 5\nprint(x)\n", None),
    ];
    for (body, want) in cases {
        let decl = &body[..body.find(" =").unwrap()];
        assert_eq!(range_of(&format!("{head}{body}"), decl), want, "{body}");
    }
    let shadowed = format!("{head}fn max(a: Int, b: Int) -> Int = a + b\nlet m = max(a, 3)\nprint(m)\n");
    assert_eq!(range_of(&shadowed, "let m"), None, "a user fn named max");
}

#[test]
fn an_if_condition_narrows_the_names_it_compares() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        ("if 3 <= a < n:\n    let x = a + 0\n    print(x)\n", Some((3, 9))),
        ("if n > 50 and a == 2:\n    let x = n * a\n    print(x)\n", Some((102, 200))),
        ("if n < 10:\n    let x = n\n    print(x)\nelse:\n    let y = n\n    print(y)\n", Some((1, 9))),
        ("if a >= 20:\n    let x = a\n    print(x)\n", None),
    ];
    for (body, want) in cases {
        assert_eq!(range_of(&format!("{head}{body}"), "let x"), want, "{body}");
    }
    let otherwise = [
        ("if n < 10:\n    print(n)\nelse:\n    let y = n\n    print(y)\n", Some((10, 100))),
        ("if n < 10:\n    print(n)\nelif a != 4:\n    print(a)\nelse:\n    let y = n * a\n    print(y)\n", Some((40, 400))),
        ("if not (n > 30 or a < 2):\n    let y = n + a\n    print(y)\n", Some((3, 39))),
        ("if a < 2 and n < 5:\n    print(n)\nelse:\n    let y = n\n    print(y)\n", Some((1, 100))),
        ("if 2 < a < 5:\n    print(a)\nelse:\n    let y = a\n    print(y)\n", Some((0, 9))),
        ("var i = 0\nwhile a < 5 and i < n:\n    let y = a\n    print(y)\n    i += 1\n", Some((0, 4))),
    ];
    for (body, want) in otherwise {
        assert_eq!(range_of(&format!("{head}{body}"), "let y"), want, "{body}");
    }
}

#[test]
fn a_comparison_of_two_names_bounds_their_difference() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\n";
    let cases = [
        ("if a < n:\n    let x = n - a\n    print(x)\n", Some((1, 100))),
        ("if a >= n:\n    let x = a - n\n    print(x)\n", Some((0, 8))),
        ("if n <= a:\n    print(n)\nelse:\n    let x = a - n\n    print(x)\n", Some((-100, -1))),
        ("if a == n:\n    let x = (n) - (a)\n    print(x)\n", Some((0, 0))),
        ("if a < n:\n    a = 5\n    let x = n - a\n    print(x)\n", Some((-8, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\nlet x = lo - hi\nprint(x)\n", Some((0, 0))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo = hi + 5\nlet x = lo - hi\nprint(x)\n", Some((0, 5))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    if a < 5:\n        lo = hi + 1\n        continue\n    lo = hi\nlet x = lo - hi\nprint(x)\n", Some((0, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo += 1\n    if lo == 3:\n        break\nlet x = lo - hi\nprint(x)\n", Some((-100, 99))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let x = hi - lo\n    print(x)\n    lo += 1\n", Some((1, 100))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    lo += 1\n    let x = hi - lo\n    print(x)\n", Some((0, 99))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    let x = hi - mid\n    print(x)\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\n", Some((1, 50))),
        ("var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    let x = mid - lo\n    print(x)\n", Some((-1, 50))),
    ];
    for (body, want) in cases {
        let src = format!("{head}{body}").replace("let a:", if body.contains("a = 5") { "var a:" } else { "let a:" });
        assert_eq!(range_of(&src, "let x"), want, "{body}");
    }
}

#[test]
fn a_for_loop_knows_how_often_it_runs() {
    let loops = |src: &str| -> Vec<(u128, Option<u128>)> {
        crate::proven_loops(src).into_iter().map(|l| (l.times, l.total)).collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let nested = format!("{head}for v in V:\n    for i in 0..q:\n        print(v, i)\n");
    assert_eq!(loops(&nested), [(200000, Some(200000)), (1000, Some(200000000))]);
    let in_fn = format!("{head}fn f(k: 1..=50):\n    for i in 0..=k:\n        print(i)\nfor v in V:\n    f(3)\n");
    assert_eq!(loops(&in_fn), [(51, Some(51)), (200000, Some(200000))]);
    let in_while = format!("{head}var t = 0\nwhile t < 5:\n    for i in 1..n:\n        print(i)\n    t += 1\n");
    assert_eq!(loops(&in_while), [(199999, None)]);
    let unknown = format!("{head}let xs = [1, 2]\nfor x in xs:\n    for i in 0..q:\n        print(x, i)\n");
    assert_eq!(loops(&unknown), [(1000, None)]);
    let counts = |src: &str| -> Vec<(String, Option<String>)> {
        crate::proven_loops(src).into_iter().map(|l| (l.count, l.total_count)).collect()
    };
    let s = |t: &str| t.to_string();
    assert_eq!(counts(&nested), [(s("n"), Some(s("n"))), (s("q"), Some(s("n * q")))]);
    let shifted = format!("{head}for v in V:\n    for i in 2..q + 1:\n        for j in 1..=q - 1:\n            print(v, i, j)\n");
    assert_eq!(counts(&shifted)[2], (s("q - 1"), Some(s("n * 999 * (q - 1)"))));
    assert_eq!(counts(&in_fn)[0], (s("51"), Some(s("51"))));
}

#[test]
fn a_fn_knows_how_many_loop_iterations_a_call_makes() {
    let fns = |src: &str| -> Vec<(String, Option<u128>)> {
        let name = |f: &crate::ProvenFn| src[f.name.0 as usize..f.name.1 as usize].to_string();
        crate::proven_fns(src).iter().filter(|f| f.steps != Some(0)).map(|f| (name(f), f.steps)).collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let src = format!(
        "{head}fn grid():\n    for v in V:\n        for i in 0..q:\n            print(v, i)\n    for i in 0..q:\n        print(i)\n\
         fn walk(k: Int):\n    var t = 0\n    while t < k:\n        t += 1\n\
         fn plain(k: Int) -> Int:\n    return k + 1\n\
         fn over(xs: List<Int>):\n    for x in xs:\n        print(x)\n"
    );
    let some = |s: &str, n| (s.to_string(), n);
    assert_eq!(
        fns(&src),
        [some("grid", Some(200000 + 200000000 + 1000)), some("walk", None), some("over", None)]
    );
    let calls = format!(
        "{src}fn twice():\n    for i in 0..2:\n        grid()\n    plain(3)\n\
         fn again(k: Int):\n    again(k - 1)\n    grid()\n\
         fn mapped():\n    let ys = [1, 2].map(x => grid())\n"
    );
    let found = fns(&calls);
    assert_eq!(found[3..], [some("twice", Some(2 + 2 * 200201000)), some("again", None), some("mapped", None)]);
    let counts: Vec<Option<String>> =
        crate::proven_fns(&calls).into_iter().filter(|f| f.steps != Some(0)).map(|f| f.count).collect();
    let s = |t: &str| Some(t.to_string());
    assert_eq!(counts, [s("n + n * q + q"), None, None, s("2 + 2 * n + 2 * n * q + 2 * q"), None, None]);
}

#[test]
fn a_fn_lists_the_calls_its_loop_count_leaves_out() {
    let src = "let n: 1..=10 = read()\nfn inner() -> Int:\n    return 1\n\
               fn outer(xs: List<Int>):\n    for i in 0..n:\n        print(inner(), xs.sorted().len)\n\
               \x20   fn nested():\n        xs.reverse()\n    (inner)()\n    let p = P(1)\n\
               struct P:\n    a: Int\n";
    let callees: Vec<Vec<&str>> = crate::proven_fns(src)
        .iter()
        .map(|f| f.callees.iter().map(|&(s, e)| &src[s as usize..e as usize]).collect())
        .collect();
    // `inner` is in the loop count; `nested` speaks for its own calls.
    assert_eq!(callees, [vec![], vec!["reverse"], vec!["print", "sorted", "(inner)"]]);
    let builds: Vec<usize> = crate::proven_fns(src).iter().map(|f| f.builds.len()).collect();
    assert_eq!(builds, [0, 0, 1]);
}

#[test]
fn the_prover_names_the_inputs_an_overflow_needs() {
    let overflows = |src: &str| -> Vec<Option<String>> {
        crate::proven_expr_ranges(src).into_iter().map(|e| e.overflow).collect()
    };
    let head = "let n: 1..=200000 = read()\nlet a: -1e9..=1e9 = read()\nlet m = n * n\n";
    let some = |s: &str| Some(s.to_string());
    assert_eq!(overflows(&format!("{head}print(m * m)\n")), [None, some("n = 55109")]);
    assert_eq!(overflows(&format!("{head}print(a * a * a)\n")), [None, None, some("a = 2097152")]);
    assert_eq!(overflows(&format!("{head}print(n * n * n)\n")), [None, None, None]);
    let narrowed = format!("{head}if n < 1000:\n    print(n * n * n * n * n)\n");
    assert!(overflows(&narrowed).iter().all(Option::is_none), "a narrowed name is not an input");
    let var = format!("{head}var t = n\nt = n * n\nprint(t * t)\n");
    assert!(overflows(&var).iter().all(Option::is_none), "a var is not followed");
}

#[test]
fn vars_that_assign_each_other_are_solved_together() {
    let search = "var lo = 0\nvar hi = n\nwhile lo < hi:\n    let mid = (lo + hi) / 2\n    if a < 5:\n        lo = mid + 1\n    else:\n        hi = mid\nprint(lo, hi)\n";
    let small = format!("let n: 1..=100 = read()\nlet a: 0..=9 = read()\n{search}");
    assert_eq!(range_of(&small, "var lo"), Some((0, 101)));
    assert_eq!(range_of(&small, "var hi"), Some((0, 100)));
    assert_eq!(range_of(&small, "let mid"), Some((0, 100)));
    let wide = small.replace("1..=100", "1..=1e18");
    assert_eq!(range_of(&wide, "var lo"), Some((0, 1_000_000_000_000_000_001)));
    let cases = [
        // `j` grows without bound, so `i`, which copies it, has no range.
        ("var i = 0\nvar j = 0\nwhile i < n:\n    j += 1\n    i = j\nprint(i)\n", "var i", None),
        // `k` is bound twice in the file, so its value may be either one.
        ("let k = 500\nvar x = 0\nfor _ in 0..n:\n    let k = a\n    x = k\nprint(x)\n", "var x", None),
        ("var x = 0\nvar y = a\nfor _ in 0..n:\n    x = y * 2\n    y = min(y, x)\nprint(x)\n", "var x", Some((0, 18))),
    ];
    for (body, decl, want) in cases {
        let src = format!("let n: 1..=100 = read()\nlet a: 0..=9 = read()\n{body}");
        assert_eq!(range_of(&src, decl), want, "{body}");
    }
}

#[test]
fn a_loop_that_exits_leaves_its_condition_false() {
    let head = "let n: 1..=100 = read()\nlet a: 0..=9 = read()\nvar x = n\n";
    let halve = "while x > 9:\n    x = x / 2\n";
    let cases = [
        (format!("{halve}let y = x\nprint(y)\n"), Some((1, 9))),
        (format!("for _ in 0..a:\n    {}    let y = x + 0\n    print(y)\n", halve.replace("\n    ", "\n        ")), Some((1, 9))),
        ("while x > 9:\n    x = x / 2\n    if a == 3:\n        break\nlet y = x\nprint(y)\n".to_string(), Some((1, 100))),
        (format!("{halve}x = 50\nlet y = x\nprint(y)\n"), Some((1, 100))),
        (format!("fn grow():\n    x = 100\n{halve}grow()\nlet y = x\nprint(y)\n"), Some((0, 100))),
        ("fn grow():\n    x = 100\nif x < 10:\n    grow()\n    let y = x\n    print(y)\n".to_string(), Some((1, 100))),
    ];
    for (body, want) in cases {
        assert_eq!(range_of(&format!("{head}{body}"), "let y"), want, "{body}");
    }
}

/// A binary search over a list it never resizes keeps `0 <= lo <= mid <
/// hi <= ys.len`, so `ys[mid]` is in bounds; so is `ys[i]` under
/// `for i in 0..ys.len`. A list that is pushed to, or an index past the
/// end, proves nothing.
#[test]
fn an_index_below_a_fixed_length_is_proven_in_bounds() {
    let search = "fn count(ys: List<Int>, p: Int) -> Int:\n    var (lo, hi) = (0, ys.len)\n    while lo < hi:\n        let mid = (lo + hi) / 2\n        if ys[mid] < p: lo = mid + 1\n        else: hi = mid\n    ys.len - lo\nprint(count([1, 2], 2))\n";
    let proven = |src: &str| -> Vec<(String, String)> {
        crate::proven_indexes(src)
            .into_iter()
            .map(|p| (src[p.access.0 as usize..p.access.1 as usize].to_string(), p.proof))
            .collect()
    };
    let pair = |i: &str, l: &str| (i.to_string(), l.to_string());
    assert_eq!(proven(search), [pair("ys[mid]", "0 <= lo <= mid < hi <= ys.len")]);
    let pushed = search.replace("print(count", "fn grow(ys: List<Int>):\n    ys.push(1)\nprint(count");
    assert_eq!(proven(&pushed), []);
    let past = search.replace("(0, ys.len)", "(0, ys.len + 1)");
    assert_eq!(proven(&past), []);
    let each = "fn f(xs: List<Int>) -> Int:\n    var s = 0\n    for i in 0..xs.len:\n        s += xs[i] + xs[i + 1]\n    s\nprint(f([1]))\n";
    assert_eq!(proven(each), [pair("xs[i]", "0 <= i < xs.len")]);
    // No hover or hint shows a range a `.len` bounds only by its type.
    assert!(crate::proven_ranges(search).is_empty(), "{:?}", crate::proven_ranges(search));
}

#[test]
fn a_list_is_indexed_safely_once_nothing_changes_its_length() {
    let proofs = |src: &str| -> Vec<String> { crate::proven_indexes(src).into_iter().map(|p| p.proof).collect() };
    let built = "fn f(p: Int) -> Int:
    var ys: List<Int> = []
    for i in 0..5:
        ys.push(i)
    var (lo, hi) = (0, ys.len)
    while lo < hi:
        let mid = (lo + hi) / 2
        if ys[mid] < p: lo = mid + 1
        else: hi = mid
    lo
print(f(2))
";
    assert_eq!(proofs(built), ["0 <= lo <= mid < hi <= ys.len"]);
    let popped = built.replace("else: hi = mid", "else:\n            hi = mid\n            ys.pop()");
    assert_eq!(proofs(&popped), Vec::<String>::new());
    let by_fn = built.replace("    lo\n", "    shrink()\n    lo\n").replace(
        "    var (lo",
        "    fn shrink():\n        ys.pop()\n    var (lo",
    );
    assert_eq!(proofs(&by_fn), Vec::<String>::new());
    // A change later in a loop around the search reaches it on the next turn.
    let again = "fn f(p: Int) -> Int:
    var ys: List<Int> = [1]
    var lo = 0
    for t in 0..3:
        var hi = ys.len
        while lo < hi:
            let mid = (lo + hi) / 2
            if ys[mid] < p: lo = mid + 1
            else: hi = mid
        ys.pop()
    lo
print(f(2))
";
    assert_eq!(proofs(again), ["0 <= lo <= mid < hi <= ys.len"]);
    let stale = again.replace("        var hi = ys.len\n", "").replace("    var lo = 0\n", "    var lo = 0\n    var hi = ys.len\n");
    assert_eq!(proofs(&stale), Vec::<String>::new());
    // A breadth-first queue: each read comes before the pushes after it.
    let queue = "var q: List<Int> = [0]
var head = 0
while head < q.len:
    let u = q[head]
    head += 1
    if u < 9: q.push(u + 1)
print(head)
";
    assert_eq!(proofs(queue), ["0 <= head < q.len"]);
    let late = queue.replace("    let u = q[head]\n    head += 1\n", "    q.push(1)\n    let u = q[head]\n    head += 1\n");
    assert_eq!(proofs(&late), Vec::<String>::new());
    let same = queue.replace("    let u = q[head]\n", "    let u = q.pop()! + q[head]\n");
    assert_eq!(proofs(&same), Vec::<String>::new());
    // Reading the queue between the test and the index changes nothing.
    let read = queue.replace("    let u = q[head]\n", "    let seen = q.contains(9)\n    let u = q[head]\n");
    assert_eq!(proofs(&read), ["0 <= head < q.len"]);
    // Reordering in place keeps the length; emptying it does not.
    let sorted = queue.replace("    let u = q[head]\n", "    q.sort()\n    q.reverse()\n    let u = q[head]\n");
    assert_eq!(proofs(&sorted), ["0 <= head < q.len"]);
    let cleared = queue.replace("    let u = q[head]\n", "    q.clear()\n    let u = q[head]\n");
    assert_eq!(proofs(&cleared), Vec::<String>::new());
    assert!([&late, &same].iter().all(|src| cpl_syntax::parse_file(src).diagnostics.is_empty()));
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../proto/ports/scout_batch_14/noi19p1.cp");
    let port = &std::fs::read_to_string(path).unwrap();
    let routes: Vec<String> = crate::proven_indexes(port)
        .into_iter()
        .map(|p| format!("{} {}", &port[p.access.0 as usize..p.access.1 as usize], p.proof))
        .collect();
    assert_eq!(routes[..2], ["routes[i] 0 <= i < routes.len", "routes[i] 0 <= i < routes.len"]);
}

#[test]
fn a_row_is_indexed_safely_below_its_own_length() {
    let proofs = |src: &str| -> Vec<String> {
        crate::proven_indexes(src)
            .into_iter()
            .map(|p| format!("{} {}", &src[p.access.0 as usize..p.access.1 as usize], p.proof))
            .collect()
    };
    let walk = "fn f(g: List<List<Int>>) -> Int:
    var s = 0
    for u in 0..g.len:
        for j in 0..g[u].len:
            s += g[u][j]
    s
print(f([[1], [2, 3]]))
";
    let rows = ["g[u] 0 <= u < g.len", "g[u] 0 <= u < g.len", "g[u][j] 0 <= j < g[u].len"];
    assert_eq!(proofs(walk), rows);
    let grown = walk.replace("            s += g[u][j]", "            s += g[u][j]\n            g[u].push(1)");
    assert_eq!(proofs(&grown), ["g[u] 0 <= u < g.len"; 3]);
    let other = walk.replace("s += g[u][j]", "s += g[u][j]\n            u = 0");
    assert_eq!(proofs(&other), ["g[u] 0 <= u < g.len"]);
    let cell = walk.replace("s += g[u][j]", "g[u][j] = s");
    assert_eq!(proofs(&cell), rows);
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../proto/ports/scout_batch_33/ioi12p6.cp");
    let port = &std::fs::read_to_string(path).unwrap();
    assert!(proofs(port).contains(&"kids[v][i] 0 <= i < kids[v].len".to_string()), "{:?}", proofs(port));
}


#[test]
fn a_method_call_knows_the_size_of_an_array_receiver() {
    let sizes = |src: &str| -> Vec<(String, u128, String)> {
        crate::proven_sizes(src)
            .into_iter()
            .map(|s| (src[s.receiver.0 as usize..s.receiver.1 as usize].to_string(), s.max, s.count))
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\ndomain R = 0..3\n";
    let src = format!(
        "{head}let a: [0..q]Int = read()\nfn f(b: [V, R]Int, c: List<Int>):\n    print(a.sorted(), b.sorted(), c.sorted())\n\
         fn g(a: List<Int>):\n    print(a.sorted())\n"
    );
    let s = |t: &str| t.to_string();
    assert_eq!(sizes(&src), [(s("a"), 1000, s("q")), (s("b"), 600000, s("n * 3"))]);
}

#[test]
fn a_list_only_pushed_in_counted_loops_has_a_bounded_size() {
    let sizes = |src: &str| -> Vec<(String, u128, String)> {
        crate::proven_sizes(src)
            .into_iter()
            .map(|s| (src[s.receiver.0 as usize..s.receiver.1 as usize].to_string(), s.max, s.count))
            .collect()
    };
    let head = "let n: 1..=200000 = read()\nlet q: 1..=1000 = read()\ndomain V = 1..=n\n";
    let src = format!(
        "{head}fn f():\n    var xs: List<Int> = []\n    for v in V:\n        xs.push(v)\n        for j in 0..q:\n            xs.push(j)\n    xs.push(0)\n    xs.pop()\n    print(xs.sorted())\n\
         fn g():\n    var ys: List<Int> = []\n    var i = 0\n    while i < n:\n        ys.push(i)\n        i += 1\n    print(ys.sorted())\n\
         fn h():\n    var zs: List<Int> = []\n    zs.push(1)\n    zs.extend([1, 2])\n    print(zs.sorted())\n\
         fn k():\n    var ws: List<Int> = []\n    let add = (x: Int) => ws.push(x)\n    add(1)\n    print(ws.sorted())\n"
    );
    let s = |t: &str| t.to_string();
    let got = sizes(&src);
    assert_eq!(got.iter().rfind(|(r, ..)| r == "xs"), Some(&(s("xs"), 200200001, s("n + n * q + 1"))));
    assert!(got.iter().all(|(r, ..)| r == "xs"), "{got:?}");
}

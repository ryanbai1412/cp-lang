use crate::cli;
use std::path::{Path, PathBuf};

/// `testdata/library-lint` copied to a fresh directory: a program using a
/// library whose fn declares `@lint`.
fn fixture(name: &str) -> PathBuf {
    let from = Path::new(env!("CARGO_MANIFEST_DIR")).join("testdata/library-lint");
    let to = std::env::temp_dir().join(format!("cpl-library-lint-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&to);
    for rel in ["cp.toml", "main.cp", "mylib/lib.toml", "mylib/geo.cp"] {
        std::fs::create_dir_all(to.join(rel).parent().unwrap()).unwrap();
        std::fs::copy(from.join(rel), to.join(rel)).unwrap();
    }
    to
}

/// A library's `@lint` fires in a program that uses it, keeps quiet where the
/// two sides differ, where a repeated side calls something, and under
/// `with lint(name: allow)`; `cpl fix` writes the call.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_library_lint_fires_and_fixes_in_a_program_using_the_library() {
    let dir = fixture("fires");
    let main = dir.join("main.cp").to_string_lossy().into_owned();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    cli::run(std::slice::from_ref(&main), &mut out, &mut err);
    let report = String::from_utf8_lossy(&out).into_owned() + &String::from_utf8_lossy(&err);
    assert_eq!(
        report.matches("warning[square_by_hand]").count(),
        2,
        "{report}"
    );
    assert!(
        report.contains("write `geo.sq(xa - xb)`") && report.contains("write `geo.sq(xa)`"),
        "{report}"
    );
    assert!(
        report.contains("`mylib.geo` declares this lint"),
        "{report}"
    );
    cli::run_fix(
        std::slice::from_ref(&main),
        &mut Vec::new(),
        &mut Vec::new(),
    );
    let fixed = std::fs::read_to_string(&main).unwrap();
    assert!(
        fixed.contains("let d2 = geo.sq(xa - xb)\n") && fixed.contains("let f = geo.sq(xa) + 1\n"),
        "{fixed}"
    );
    assert!(
        fixed.contains("let e = xa * xb\n") && fixed.contains("let g: Int = read() * read()\n"),
        "{fixed}"
    );
    assert!(fixed.contains("print(d2, e, f, g, xb * xb)"), "{fixed}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A hole after `.` is a field name, and a lint with no fix may
/// leave out of `to` a hole that matched a call (nothing is rewritten).
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_hole_after_a_dot_names_a_field_and_an_unfixed_lint_drops_calls() {
    let decl = "@lint(hand_rolled_string_algo, from: \"t[v].f[c] ??= w\\nv = t[v].f[c]!\", to: \"Tr.make()\", holes: \"t, v, f, c, w\")\npub fn make() -> Int = 0\n@lint(hand_rolled_string_algo, from: \"x * x\", to: \"sq(x)\")\npub fn sq(x: Int) -> Int = x * x\nstruct N:\n    kid: [0..2]?Int\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "L0516" && d.message.starts_with("write"))
            .map(|d| d.message)
            .filter(|m| !m.contains("Trie.new"))
            .collect()
    };
    let walk = "var ns: List<N> = [N([0..2] _ => None)]\nvar o = 0\nfor b in [1, 0]:\n    ns[o].kid[b] ??= ns.push_id(N([0..2] _ => None))\n    o = ns[o].kid[b]!\nlet y = 3\nprint(o, y * y)\n";
    assert_eq!(found(walk), ["write `Tr.make()`", "write `sq(y)`"]);
    let other = walk.replace("o = ns[o].kid[b]!", "o = ns[0].kid[b]!");
    assert_eq!(found(&other), ["write `sq(y)`"]);
}

/// A hole matches a type as it does an expression: the domain of a
/// `Fenwick<C, Int>()` the call no longer needs. Rule: R06.4.9.
#[test]
fn a_hole_matches_a_type() {
    let decl = "@lint(tree_by_hand, from: \"var t = Fenwick<C, Int>()\\nt.add(x, 1)\", to: \"let t = once(x)\", holes: \"C\")\npub fn once<K>(x: K) -> Int = 0\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "tree_by_hand")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let hit =
        "domain D = 0..3\nlet k = D.first!\nvar f = Fenwick<D, Int>()\nf.add(k, 1)\nprint(f)\n";
    assert_eq!(found(hit), ["let f = once(k)"]);
    let other =
        "domain D = 0..3\nlet k = D.first!\nvar f = Fenwick<D, Bool>()\nf.add(k, 1)\nprint(f)\n";
    assert!(found(other).is_empty());
}

/// `cp.toml`'s `[lints]` sets a library lint's level like a built-in one's,
/// without calling it unknown; a name no library declares still is.
/// Rule: lints/R19.
#[test]
fn cp_toml_sets_a_library_lints_level() {
    let dir = fixture("level");
    let toml = dir.join("cp.toml");
    let mut text = std::fs::read_to_string(&toml).unwrap();
    text += "\n[lints]\nsquare_by_hand = \"deny\"\nnope = \"allow\"\n";
    std::fs::write(&toml, text).unwrap();
    let main = dir.join("main.cp").to_string_lossy().into_owned();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    cli::run(std::slice::from_ref(&main), &mut out, &mut err);
    let report = String::from_utf8_lossy(&out).into_owned() + &String::from_utf8_lossy(&err);
    assert_eq!(
        report.matches("error[square_by_hand]").count(),
        2,
        "{report}"
    );
    assert!(
        !report.contains("no lint called `square_by_hand`"),
        "{report}"
    );
    assert!(report.contains("no lint called `nope`"), "{report}");
}

/// A `from` may be a run of statements: it matches consecutive statements of
/// one block, whole, with each bound name the same across the run, and its
/// fix replaces the run. A wider finding it rewrites (L0521) gives way.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_library_lint_matches_a_run_of_statements() {
    let decl = "@lint(totals, from: \"var p = [0]\\nfor x in xs:\\n    p.push(p.last! + x)\", to: \"let p = totals(xs)\")\npub fn totals(xs: List<Int>) -> List<Int> = xs\n";
    let found = |body: &str| -> Vec<(String, String)> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .map(|d| {
                let fix = d.fixes.first().map(|f| f.edits[0].1.clone());
                (d.code.0.to_string(), fix.unwrap_or_default())
            })
            .collect()
    };
    let run =
        "let a = [3, 1, 4]\nvar s = [0]\nfor v in a:\n    s.push(s.last! + v)\nprint(s.at(1))\n";
    // std declares the same run on `scan` (decision 512).
    assert_eq!(
        found(run),
        [
            ("prefix_totals_list".into(), "let s = a.scan(+)".into()),
            ("totals".into(), "let s = totals(a)".into())
        ]
    );
    let nested = "fn f(a: List<Int>):\n    var s = [0]\n    for v in a:\n        s.push(s.last! + v)\n    print(s.len)\nf([1])\n";
    assert!(found(nested).contains(&("totals".into(), "let s = totals(a)".into())));
    // Another statement between, or a different list pushed to, is not the run.
    let apart = "let a = [3, 1, 4]\nvar s = [0]\nprint(a)\nfor v in a:\n    s.push(s.last! + v)\n";
    let other =
        "let a = [3, 1, 4]\nvar s = [0]\nvar t = [0]\nfor v in a:\n    t.push(s.last! + v)\n";
    for near in [apart, other] {
        assert!(
            found(near).iter().all(|(c, _)| !c.contains("totals")),
            "{near}"
        );
    }
}

/// `reads: "p[k] -> p.at(k)"` rewrites, with the fix, every later read of
/// a name `from` declares; a later use that is no such read (a write,
/// `p.len`) leaves the finding without a fix.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn reads_rewrites_the_later_reads_of_a_name_from_declares() {
    let decl = "@lint(table_by_hand, from: \"var p = List.filled(xs.len + 1, 0)\\nfor i in 0..xs.len:\\n    p[i + 1] = p[i] + xs[i]\", to: \"let p = table(xs)\", reads: \"p[k] -> p.at(k)\", holes: \"k\")\npub fn table(xs: List<Int>) -> List<Int> = xs\n";
    let fixed = |body: &str| -> (usize, String) {
        let src = format!("{decl}{body}");
        let found: Vec<_> = (crate::lint(&src, &crate::Config::default()).into_iter())
            .filter(|d| d.code.0 == "table_by_hand")
            .collect();
        let (text, _) = cpl_diag::splice_fixes(&src, &found);
        (found.len(), text[decl.len()..].to_string())
    };
    let head = "let a = [1, 2]\nvar p = List.filled(a.len + 1, 0)\nfor i in 0..a.len:\n    p[i + 1] = p[i] + a[i]\n";
    assert_eq!(
        fixed(&format!("{head}print(p[2] - p[a[0]])\nprint(p[1])\n")),
        (
            1,
            "let a = [1, 2]\nlet p = table(a)\nprint(p.at(2) - p.at(a[0]))\nprint(p.at(1))\n"
                .into()
        )
    );
    for unread in ["p[0] = 5\nprint(p[1])\n", "print(p.len)\n", "print(p)\n"] {
        let body = format!("{head}{unread}");
        assert_eq!(fixed(&body), (1, body.clone()), "{unread}");
    }
}

/// A hole `to` puts in the initializer of a name it declares (`let p =
/// table(..)`) does not match code that reads that name.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_declared_name_s_initializer_does_not_read_the_name() {
    let decl = "@lint(table_by_hand, from: \"for i in 0..n:\\n    p[i + 1] = p[i] + e\", advice: \"let p = table([e for i in 0..n])\", holes: \"n, p, e\")\npub fn table(xs: List<Int>) -> List<Int> = xs\n";
    let found = |step: &str| {
        let src = format!("{decl}let a = [1, 2]\nvar q = List.filled(3, 0)\nfor i in 0..2:\n    q[i + 1] = q[i] + {step}\nprint(q)\n");
        (crate::lint(&src, &crate::Config::default()).into_iter())
            .filter(|d| d.code.0 == "table_by_hand")
            .count()
    };
    assert_eq!(found("a[i]"), 1);
    assert_eq!(found("q[i] * 2"), 0);
}

/// `holes:` names more holes than the fn's parameters; `to` may drop one
/// when it matched code that calls nothing, `to` may use the fn as a getter
/// (`xs.walk`), and a lint named like a `spec/lints` record reports as it.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn declared_holes_match_free_names_and_a_record_name_keeps_its_code() {
    let found = |name: &str, body: &str| -> Vec<(String, String)> {
        let decl = format!(
            "@lint({name}, from: \"for i in 0..n:\\n    print(xs[i])\", to: \"for x in xs.walk:\\n    print(x)\", holes: \"n\")\npub fn walk(xs: List<Int>) -> List<Int> = [x for x in xs]\n"
        );
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "L0557" || d.code.0 == name)
            .map(|d| (d.code.0.to_string(), d.message.clone()))
            .collect()
    };
    let hit = "let a = [3, 1]\nlet m = a.len\nfor j in 0..m:\n    print(a[j])\n";
    assert_eq!(
        found("binom_by_hand", hit),
        [(
            "L0557".into(),
            "write `for x in a.walk:\n    print(x)`".into()
        )]
    );
    assert_eq!(found("walk_by_hand", hit).len(), 1);
    let calls = "let a = [3, 1]\nfn m() -> Int = 2\nfor j in 0..m():\n    print(a[j])\n";
    assert!(found("walk_by_hand", calls).is_empty());
}

/// One fn declares a lint twice, one `from` per spelling, and `from` does
/// not match an assigned place. Rule: R06.4.9.
#[test]
fn a_lint_has_spellings_and_skips_assigned_places() {
    let decl = "@lint(at_end, from: \"xs[xs.len - 1]\", to: \"end(xs)\")\n@lint(at_end, from: \"xs[-1 + xs.len]\", to: \"end(xs)\")\npub fn end(xs: List<Int>) -> Int = xs[0]\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "at_end")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let read = "var a = [3, 1]\nprint(a[a.len - 1] + a[-1 + a.len])\n";
    assert_eq!(found(read), ["end(a)", "end(a)"]);
    assert!(found("var a = [3, 1]\na[a.len - 1] = 0\nprint(a)\n").is_empty());
}

/// A lint named like a record with `no_fix` still names `to`, but offers no
/// fix: `yes(ok)` prints the judge profile's words, which may not be the
/// ones written (decision 301).
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_record_with_no_fix_keeps_a_library_lint_to_advice() {
    let decl = "@lint(bool_ternary_print, from: \"if ok: \\\"YES\\\" else: \\\"NO\\\"\", to: \"word(ok)\")\npub fn word(ok: Bool) -> Str = yes(ok)\n";
    let src = format!("{decl}let n: Int = read()\nprint(if n > 0: \"YES\" else: \"NO\")\n");
    let found: Vec<_> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.message == "write `word(n > 0)`")
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].code.0, "L0524");
    assert!(found[0].fixes.is_empty(), "{:?}", found[0].fixes);
}

/// A `fn` a `from` declares is bound, its name and its parameters, so a
/// recursive helper matches under any names. Rule: R06.4.9.
#[test]
fn a_fn_from_declares_binds_its_name_and_parameters() {
    let decl = "@lint(walk_all, from: \"fn go(u: Int):\\n    if u > 0:\\n        go(u - 1)\\ngo(n)\", to: \"walk(n)\")\npub fn walk(n: Int) = ()\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "walk_all")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let renamed = "let k = 3\nfn down(x: Int):\n    if x > 0:\n        down(x - 1)\ndown(k)\n";
    assert_eq!(found(renamed), ["walk(k)"]);
    let mixed = "let k = 3\nfn down(x: Int):\n    if x > 0:\n        go(x - 1)\nfn go(y: Int) = ()\ndown(k)\n";
    assert!(found(mixed).is_empty());
}

/// A struct's method declares a lint like a top-level fn, and `to` calls
/// it through the struct. Rule: R06.4.9.
#[test]
fn a_struct_method_declares_a_lint() {
    let decl = "struct Sq:\n    v: Int\n    @lint(sq_by_hand, from: \"x * x\", to: \"Sq.of(x)\")\n    pub fn of(x: Int) -> Int = x * x\n";
    let src = format!("{decl}let a: Int = read()\nprint(a * a)\n");
    let found: Vec<String> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "sq_by_hand")
        .map(|d| d.fixes[0].edits[0].1.clone())
        .collect();
    assert_eq!(found, ["Sq.of(a)"]);
}

/// `when:` keeps a finding only where what a hole matched is proven in
/// bounds: an integer literal, or a range type whose ends lie in them.
/// Rule: R06.4.9.
#[test]
fn when_keeps_a_finding_only_where_the_hole_is_proven_in_bounds() {
    let decl = "@lint(up, from: \"(self + b - 1) / b\", to: \"up(self, b)\", when: \"b >= 1\")\npub fn up(self: Int, b: Int) -> Int = self\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "up")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let src = "let n: 1..=100 = read()\nlet k: 1..=10 = read()\nlet m: 0..=10 = read()\nlet j: Int = read()\nprint((n + k - 1) / k, (n + 4 - 1) / 4, (n + m - 1) / m, (n + j - 1) / j, (n + 0 - 1) / 0)\n";
    assert_eq!(found(src), ["up(n, k)", "up(n, 4)"]);
}

/// A run ending in a block is replaced up to its last token, so the fix
/// keeps the next statement's line and indent. Rule: R06.4.9.
#[test]
fn a_run_ending_in_a_block_is_replaced_up_to_its_last_token() {
    let decl = "@lint(totals, from: \"var p = [0]\\nfor x in xs:\\n    p.push(p.last! + x)\", to: \"let p = totals(xs)\")\npub fn totals(xs: List<Int>) -> Int:\n    xs.len\n";
    let src = format!(
        "{decl}fn f(a: List<Int>):\n    var s = [0]\n    for v in a:\n        s.push(s.last! + v)\n    print(s.len)\nf([1])\n"
    );
    let spans: Vec<&str> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "totals")
        .map(|d| &src[d.fixes[0].edits[0].0.start as usize..d.fixes[0].edits[0].0.end as usize])
        .collect();
    assert_eq!(
        spans,
        ["var s = [0]\n    for v in a:\n        s.push(s.last! + v)"]
    );
}

/// A struct's method declares a lint like a top-level fn; `to` calls it as
/// a method. Rule: R06.4.9.
#[test]
fn a_method_declares_a_lint() {
    let decl = "pub struct W(pub v: Int):\n    @lint(w_by_hand, from: \"x + 1\", to: \"W(x).inc()\", holes: \"x\")\n    pub fn inc(self) -> Int = self.v + 1\n";
    let fixes: Vec<String> = crate::lint(
        &format!("{decl}let y = 3\nprint(y + 1)\n"),
        &crate::Config::default(),
    )
    .into_iter()
    .filter(|d| d.code.0 == "w_by_hand")
    .map(|d| d.fixes[0].edits[0].1.clone())
    .collect();
    assert_eq!(fixes, ["W(y).inc()"]);
}

/// A lint whose `from` calls the fn and whose `to` does not drops a needless
/// call of it, titled "drop `f`" (both leaving it uncalled is E0505,
/// `lint-to-does-not-call-the-fn.cp`). Rule: R06.4.9.
#[test]
fn a_lint_may_drop_a_needless_call_of_its_fn() {
    let decl = "@lint(peek, from: \"let x = self.top!\\nself.pop()\", to: \"let x = self.pop()!\")\n@get pub fn top(self: List<Int>) -> ?Int = self.last\n";
    let src = format!("{decl}var s = [1, 2]\nlet u = s.top!\ns.pop()\nprint(u)\n");
    let fixes: Vec<(String, String)> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "peek")
        .map(|d| (d.fixes[0].title.clone(), d.fixes[0].edits[0].1.clone()))
        .collect();
    assert_eq!(fixes, [("drop `top`".into(), "let u = s.pop()!".into())]);
}

/// A hole standing as a statement matches the rest of its block, a tuple
/// after `for` or `let` binds each name, and the fix keeps the site's
/// indentation and moves the matched statements with the hole.
/// Rule: R06.4.9, spec/core/06-control-functions.md.
#[test]
fn a_statement_hole_matches_a_block_and_the_fix_keeps_its_indentation() {
    let decl = "@lint(near, from: \"for (dx, dy) in [(-1, 0), (1, 0)]:\\n    let (nx, ny) = (x + dx, y + dy)\\n    if nx >= 0 and nx < h:\\n        body\", to: \"for (nx, ny) in near(x, y):\\n    body\", holes: \"x, y, h, body\")\npub fn near(a: Int, b: Int) -> List<(Int, Int)> = [(a, b)]\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "near")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let hit = "fn f(r: Int, c: Int, n: Int):\n    for (a, b) in [(-1, 0), (1, 0)]:\n        let (p, q) = (r + a, c + b)\n        if p >= 0 and p < n:\n            if p > 1:\n                print(p)\n            print(q)\nf(1, 2, 3)\n";
    assert_eq!(
        found(hit),
        ["for (p, q) in near(r, c):\n        if p > 1:\n            print(p)\n        print(q)"]
    );
    let other = hit.replace("(r + a, c + b)", "(r + b, c + a)");
    let more = hit.replace(
        "            print(q)\n",
        "        else:\n            print(q)\n",
    );
    for near in [other, more] {
        assert!(found(&near).is_empty(), "{near}");
    }
}

/// `to` may write the operator the fn's body is (calling nothing), a hole
/// whose parameter is written `Int` matches only an `Int`, and one fn's
/// spellings share sites, the first taking it. Rule: R06.4.9.
#[test]
fn a_lint_writes_its_fns_operator_and_holes_keep_their_scalar_type() {
    let decl = "@lint(rem_guard, from: \"(x % d + d) % d\", to: \"x % d\")\n@lint(rem_guard, from: \"(x + d) % d\", to: \"x % d\")\npub fn rem(x: Int, d: Int) -> Int = x % d\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "rem_guard")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let ints =
        "let n: 1..=9 = read()\nlet a: Int = read()\nprint((a - 1 + n) % n, (a % n + n) % n)\n";
    assert_eq!(found(ints), ["(a - 1) % n", "a % n"]);
    let floats = "let f: F64 = read()\nprint((f + 2.0) % 2.0)\n";
    assert!(found(floats).is_empty(), "{:?}", found(floats));
}

/// `to` may use the fn as its compound assignment; then every hole must call
/// nothing (R11), the hole after `=` keeps no brackets, the finding labels
/// what `to`'s first hole matched and ends at its last token, and a comment
/// inside the match leaves it without a fix. Rule: R06.4.9.
#[test]
fn a_compound_to_needs_pure_holes_and_a_comment_drops_the_fix() {
    let decl = "@lint(keep_max, from: \"x = if e > x: e else: x\", to: \"x max= e\", holes: \"x, e\")\n@lint(keep_max, from: \"if not (x >= e): x = e\", to: \"x max= e\", holes: \"x, e\")\npub fn max(a: Int, b: Int) -> Int = if a > b: a else: b\n";
    let found = |body: &str| -> Vec<(String, String, Option<String>)> {
        let src = format!("{decl}{body}");
        crate::lint(&src, &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "keep_max")
            .map(|d| {
                let label = d.labels[0].span.text(&src).to_string();
                let fix = d.fixes.first().map(|f| f.edits[0].1.clone());
                (d.span.text(&src).to_string(), label, fix)
            })
            .collect()
    };
    let hit = "fn bump() -> Int = 1\nvar best = 0\nfor v in [3, 1]:\n    best = if v + 1 > best: v + 1 else: best\n    best = if bump() > best: bump() else: best\n    if not (best >= v):\n        best = v\n    if not (best >= v):  // a comment\n        best = v\nprint(best)\n";
    assert_eq!(
        found(hit),
        [
            (
                "best = if v + 1 > best: v + 1 else: best".into(),
                "best".into(),
                Some("best max= v + 1".into())
            ),
            (
                "if not (best >= v):\n        best = v".into(),
                "best".into(),
                Some("best max= v".into())
            ),
            (
                "if not (best >= v):  // a comment\n        best = v".into(),
                "best".into(),
                None
            ),
        ]
    );
}

/// A hole does not match a bare generator: `total(x * x for x in a)` folds
/// the generator, and `(x * x for x in a).total()` is no value to call on.
/// Rule: R06.4.9.
#[test]
fn a_hole_does_not_match_a_bare_generator() {
    let decl = "@lint(total_free, from: \"total(xs)\", to: \"xs.total()\")\npub fn total(xs: List<Int>) -> Int = xs.len\n";
    let src = format!(
        "{decl}let a = [3, 1]\nprint(total(a), total(x * x for x in a), total([x for x in a]))\n"
    );
    let fixes: Vec<String> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "total_free")
        .map(|d| d.fixes[0].edits[0].1.clone())
        .collect();
    assert_eq!(fixes, ["a.total()", "[x for x in a].total()"]);
}

/// A hole written as a `let` pattern matches any pattern there (and a
/// type hole any type), and a second use compares tokens, so a tuple read and
/// pushed whole matches but a shifted push does not. Rule: R06.4.9.
#[test]
fn a_hole_matches_a_binding_pattern() {
    let decl = "@lint(read_each, from: \"var xs: L = []\\nfor _ in 0..n:\\n    let v: T = read()\\n    xs.push(v)\", to: \"let xs: [0..n]T = take(n)\", holes: \"L, v, T\")\npub fn take(n: Int) -> Int = n\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "read_each")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let pair = "let m: 1..=9 = read()\nvar es: List<(Int, Int)> = []\nfor _ in 0..m:\n    let (u, w): (1..=9, Int) = read()\n    es.push((u, w))\nprint(es.len)\n";
    assert_eq!(found(pair), ["let es: [0..m](1..=9, Int) = take(m)"]);
    let one = "var ks: List<Int> = []\nfor _ in 0..3:\n    let k: Int = read()\n    ks.push(k)\nprint(ks.len)\n";
    assert_eq!(found(one), ["let ks: [0..3]Int = take(3)"]);
    let shifted = pair.replace("es.push((u, w))", "es.push((u - 1, w))");
    assert!(found(&shifted).is_empty(), "{shifted}");
}

/// `from:` in `to` is a label, not the fn's `from`; a record `on` heaps
/// keeps the finding only where the receiver is a heap; and only with no
/// fix may `to` drop a statement hole whose statements call code.
/// Rule: R06.4.9.
#[test]
fn a_no_fix_lint_drops_a_statement_hole_and_keeps_its_on_types() {
    let lint = |name: &str| {
        format!(
            "@lint({name}, from: \"while let (d, u) = pq.pop():\\n    if d > dist[u]:\\n        continue\\n    rest\", to: \"settle(pq, from: 0)\", holes: \"dist, rest\")\npub fn settle(pq: MinHeap<(Int, Int)>, from: Int) = ()\n"
        )
    };
    let body = |q: &str| {
        format!(
            "var pq = {q}\npq.push((0, 1))\nlet best = [0, 0]\nwhile let (c, v) = pq.pop():\n    if c > best[v]:\n        continue\n    print(v)\n"
        )
    };
    let found = |name: &str, q: &str| -> Vec<(String, usize)> {
        crate::lint(
            &format!("{}{}", lint(name), body(q)),
            &crate::Config::default(),
        )
        .into_iter()
        .filter(|d| d.message == "write `settle(pq, from: 0)`")
        .map(|d| (d.code.0.to_string(), d.fixes.len()))
        .collect()
    };
    let heap = "MinHeap<(Int, Int)>()";
    assert_eq!(found("stale_entry_skip", heap), [("L0553".to_string(), 0)]);
    assert!(found("stale_entry_skip", "[(0, 1)]").is_empty());
    assert!(found("settle_all", heap).is_empty());
}

/// With no fix, a finding ends before its statement hole: those statements
/// stay, so the findings in them stand. Rule: R06.4.9.
#[test]
fn a_no_fix_finding_ends_before_its_statement_hole() {
    let decl = "@lint(manual_mex, from: \"while i < q.len:\\n    let v = q[i]\\n    i += 1\\n    rest\", to: \"walk(q)\", holes: \"i, rest\")\npub fn walk(q: List<Int>) -> Int = 0\n";
    let src = format!(
        "{decl}var q = [1]\nvar i = 0\nwhile i < q.len:\n    let v = q[i]\n    i += 1\n    print(v)\n"
    );
    let spans: Vec<String> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.message == "write `walk(q)`")
        .map(|d| d.span.text(&src).to_string())
        .collect();
    assert_eq!(spans, ["while i < q.len:\n    let v = q[i]\n    i += 1"]);
}

/// Std's `@lint`s are read from its outline, so they reach a program that
/// loads none of their module: hand-written code is what does without it.
/// Rule: R06.4.9.
#[test]
fn std_lints_are_read_from_std_outline() {
    let std = crate::library::std_lints();
    assert!(
        std.iter().any(|(m, d)| m == "std.prefix"
            && d.name == "prefix_totals_list"
            && d.fn_name == "scan"),
        "{std:?}"
    );
    let src = "let xs = [3, 1]\nvar p = [0]\nfor x in xs:\n    p.push(p.last! + x)\nprint(p)\n";
    let found = crate::lint(src, &crate::Config::default());
    assert!(
        found.iter().any(|d| d.code.0 == "prefix_totals_list"),
        "{found:?}"
    );
}

/// A fix that declares again a name the code declared with a written type
/// stands only where the rest of the block uses the name through a member
/// or an index and the fixed file checks as before. Rule: R06.4.9.
#[test]
fn a_redeclaring_fix_needs_member_uses_that_still_check() {
    let decl = "@lint(fill_by_hand, from: \"var m: Map<K, Int> = Map()\\nfor x in xs:\\n    m[x] += 1\", to: \"let m = fill(xs)\", holes: \"K\")\npub fn fill(xs: List<Int>) -> Counter<Int, Int> = xs.tally()\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "fill_by_hand")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let fill = "let a = [3, 1, 3]\nvar c: Map<Int, Int> = Map()\nfor v in a:\n    c[v] += 1\n";
    assert_eq!(
        found(&format!("{fill}print(c[3], c.len)\n")),
        ["let c = fill(a)"]
    );
    // Whole, `c` prints in another order; `.first` is not a `Counter`'s.
    for after in ["print(c)\n", "print(c.first)\n"] {
        assert!(found(&format!("{fill}{after}")).is_empty(), "{after}");
    }
}

/// A `to` may be a run of statements, as a `from` may; its later lines take
/// the indent of the code it replaces. Rule: R06.4.9.
#[test]
fn a_to_run_replaces_a_from_run_at_its_indent() {
    let decl = "@lint(bump_by_hand, from: \"var m: Map<T, Int> = Map()\\nfor y in xs:\\n    m[x] = (m.get(x) ?? 0) + 1\", to: \"var m = Counter<T>()\\nfor y in xs:\\n    m.bump(x)\", holes: \"T, xs\")\npub fn bump(mut self: Counter<Int, Int>, x: Int):\n    self.push(x)\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "bump_by_hand")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let nested = "fn f(a: List<Int>):\n    var c: Map<Int, Int> = Map()\n    for v in a:\n        c[v] = (c.get(v) ?? 0) + 1\n    print(c[3])\nf([1])\n";
    assert_eq!(
        found(nested),
        ["var c = Counter<Int>()\n    for v in a:\n        c.bump(v)"]
    );
}

/// A hole `from` evaluates once per item of a comprehension and `to` once
/// matches only code that calls nothing: `rand()` per cell is not one value.
/// Rule: R06.4.9.
#[test]
fn a_hole_from_runs_per_item_and_to_runs_once_calls_nothing() {
    let decl = "@lint(grid_by_hand, from: \"[[c for _ in 0..m] for _ in 0..n]\", to: \"[n, m].filled(c)\", holes: \"c, m, n\", builtin: \"filled\")\npub fn rows(g: List<Int>) -> Int = 0\n";
    let found = |body: &str| -> Vec<String> {
        let src = format!("{decl}{body}");
        crate::lint(&src, &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "grid_by_hand")
            .map(|d| d.span.text(&src).to_string())
            .collect()
    };
    let hit = "let n = 3\nlet g = [[0 for _ in 0..4] for _ in 0..n + 1]\nprint(g)\n";
    assert_eq!(found(hit), ["[[0 for _ in 0..4] for _ in 0..n + 1]"]);
    let calls = "fn r() -> Int = 2\nlet a = [[r() for _ in 0..4] for _ in 0..3]\nlet b = [[0 for _ in 0..r()] for _ in 0..3]\nlet c = [[0 for _ in 0..4] for _ in 0..r()]\nprint(a, b, c)\n";
    assert_eq!(found(calls), ["[[0 for _ in 0..4] for _ in 0..r()]"]);
}

/// `when:` proves a constant expression by its value: `10**18` and
/// `(10**9)` are at least `10**9`, `-10**18` and `5` are not.
/// Rule: R06.4.9.
#[test]
fn when_proves_a_constant_expression_by_its_value() {
    let decl = "@lint(big, from: \"print(v)\", to: \"show(v)\", when: \"v >= 1000000000\")\npub fn show(v: Int) = print(v)\n";
    let found: Vec<String> = crate::lint(
        &format!("{decl}print(10**18)\nprint(-10**18)\nprint((10**9))\nprint(5)\n"),
        &crate::Config::default(),
    )
    .into_iter()
    .filter(|d| d.code.0 == "big")
    .map(|d| d.fixes[0].edits[0].1.clone())
    .collect();
    assert_eq!(found, ["show(10**18)", "show(10**9)"]);
}

/// `also:` keeps a finding only where one of its expressions matches in the
/// file with `from`'s holes and names as they matched, and `advice:` names
/// code without a fix and without calling the fn. Rule: R06.4.9.
#[test]
fn also_needs_a_use_and_advice_offers_no_fix() {
    let decl = "@lint(probe, from: \"var t = [v for i in r]\", advice: \"var t = [None for i in r]\", also: \"t[k] == v\\nv == t[k]\", holes: \"r, k\")\npub fn probe(v: Int) = ()\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "probe")
            .inspect(|d| assert!(d.fixes.is_empty(), "{:?}", d.fixes))
            .map(|d| d.message)
            .collect()
    };
    let compared = "var memo = [-1 for _ in 0..5]\nmemo[2] = 4\nprint(-1 == memo[2])\n";
    assert_eq!(
        found(compared),
        ["write `var memo = [None for _ in (0..5)]`"]
    );
    assert!(found("var memo = [-1 for _ in 0..5]\nmemo[2] = 4\nprint(memo[2])\n").is_empty());
    assert!(found(
        "var memo = [-1 for _ in 0..5]\nvar other = [0]\nprint(memo[0] == 0, other[0] == -1)\n"
    )
    .is_empty());
}

/// std's `advice:` lints apply to a program that loads no name of their
/// module: sentinel_table is std.opt's, and this file names nothing of it.
/// Rule: R06.4.9.
#[test]
fn std_advice_applies_without_its_module() {
    let src = "var memo: [0..=9]Int = -1\nmemo[1] = 2\nprint(memo[3] == -1)\n";
    let found: Vec<String> = crate::lint(src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "L0530")
        .map(|d| d.message)
        .collect();
    assert_eq!(found, ["write `var memo: [0..=9]?Int = None`"]);
}

/// A named argument's label in `from` and `to` is spelled as written, even
/// when it is also a parameter's name: `by:` before the hole `by`.
/// Rule: R06.4.9.
#[test]
fn a_label_in_from_is_spelled_as_written_not_a_hole() {
    let decl = "@lint(top_by, from: \"self.sorted(by: by).reversed()\", to: \"top(self, by: by)\")\npub fn top(self: List<Int>, by: Int -> Int) -> List<Int> = self\n";
    let found = |body: &str| -> Vec<String> {
        crate::lint(&format!("{decl}{body}"), &crate::Config::default())
            .into_iter()
            .filter(|d| d.code.0 == "top_by")
            .map(|d| d.fixes[0].edits[0].1.clone())
            .collect()
    };
    let src = "let a: List<Int> = read()\nprint(a.sorted(by: x => x % 7).reversed())\n";
    assert_eq!(found(src), ["top(a, by: x => x % 7)"]);
}

/// A wider finding whose fix keeps the code a library lint is about (a
/// hand pop loop becomes `sum(b.pop()! for ..)`, still popping by hand)
/// shows both.
/// Rule: R06.4.9.
#[test]
fn a_library_lint_the_wider_fix_keeps_is_shown_with_it() {
    let decl = "pub struct Bag:\n    pub v: List<Int>\n\n    @lint(bag_drain, from: \"for _ in 0..k:\\n    s += self.pop()!\", to: \"s += self.drain().take(k).sum()\", holes: \"k, s\")\n    @lint(bag_drain, from: \"sum(self.pop()! for _ in 0..k)\", to: \"self.drain().take(k).sum()\", holes: \"k\")\n    pub fn drain(mut self) -> yields Int:\n        while let x = self.pop(): yield x\n\n    pub fn pop(mut self) -> ?Int = self.v.pop()\n\n";
    let src = format!(
        "{decl}var b = Bag(v: [1, 2])\nvar t = 0\nfor _ in 0..2:\n    t += b.pop()!\nprint(t)\n"
    );
    let codes: Vec<&str> = (crate::lint(&src, &crate::Config::default()).iter())
        .map(|d| d.code.0)
        .collect();
    assert!(
        codes.contains(&"L0504") && codes.contains(&"bag_drain"),
        "{codes:?}"
    );
}

/// A method's `@lint` holds only where `self` matched a value of its type;
/// another type's method of the same name may share the lint's name. Rule: R06.4.9.
#[test]
fn a_method_lint_fires_only_on_its_own_type() {
    let src = "pub struct Cell(v: Int):\n    @get pub fn get(self) -> Int = v\n    @lint(next_by_hand, from: \"self.get + 1\", to: \"self.next()\")\n    pub fn next(self) -> Int = v + 1\npub struct Other(v: Int):\n    @get pub fn get(self) -> Int = v\n    @lint(next_by_hand, from: \"self.get + 2\", to: \"self.next()\")\n    pub fn next(self) -> Int = v + 2\nlet c = Cell(1)\nlet o = Other(2)\nprint(c.get + 1, o.get + 1, o.get + 2)\n";
    let found: Vec<String> = crate::lint(src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "next_by_hand")
        .map(|d| d.fixes[0].edits[0].1.clone())
        .collect();
    assert_eq!(found, ["c.next()", "o.next()"]);
}

/// A library's lint keeps out of the program's own fn of its fn's name: the
/// fix would make that fn call itself. Rule: R06.4.9.
#[test]
fn a_library_lint_keeps_out_of_the_program_s_fn_of_its_name() {
    let dir = fixture("own-fn");
    let src =
        "use mylib.geo\nfn sq(y: Int) -> Int = y * y\nlet v: Int = read()\nprint(sq(v), v * v)\n";
    std::fs::write(dir.join("main.cp"), src).unwrap();
    let main = dir.join("main.cp").to_string_lossy().into_owned();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    cli::run(std::slice::from_ref(&main), &mut out, &mut err);
    let report = String::from_utf8_lossy(&out).into_owned() + &String::from_utf8_lossy(&err);
    assert_eq!(
        report.matches("warning[square_by_hand]").count(),
        1,
        "{report}"
    );
    assert!(report.contains(":4:"), "{report}");
}

/// A lint with no fix still reports inside the program's own fn of its fn's
/// name: advice cannot make that fn call itself (a hand-rolled `find` is
/// what `Dsu.find` replaces). Rule: R06.4.9.
#[test]
fn a_no_fix_lint_reports_in_the_program_s_fn_of_its_name() {
    let decl = "@lint(bool_ternary_print, from: \"if ok: \\\"YES\\\" else: \\\"NO\\\"\", to: \"word(ok)\")\npub fn word(ok: Bool) -> Str = yes(ok)\n";
    let src = format!(
        "{decl}fn word(n: Int) -> Str = if n > 0: \"YES\" else: \"NO\"\nprint(word(read()))\n"
    );
    let found: Vec<_> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.message == "write `word(n > 0)`")
        .collect();
    assert_eq!(found.len(), 1, "{found:?}");
}

/// A hole `to` writes as a comprehension's item, `[e for i in 0..n]`, takes
/// the matched code bare, as the brackets delimit it; code that has its own
/// `for`, `if` or `=>` keeps its parentheses. Rule: R06.4.9.
#[test]
fn a_comprehension_item_hole_takes_its_code_bare() {
    let decl = "@lint(table_by_hand, from: \"for i in 0..n:\\n    p[i + 1] = p[i] + e\", to: \"p = table([e for i in 0..n])\", holes: \"n, p, e\")\npub fn table(xs: List<Int>) -> List<Int> = xs\n";
    let fixed = |e: &str| {
        let src = format!("{decl}let a = [1, 2]\nvar p = [0, 0, 0]\nfor i in 0..2:\n    p[i + 1] = p[i] + {e}\nprint(p)\n");
        let found: Vec<_> = (crate::lint(&src, &crate::Config::default()).into_iter())
            .filter(|d| d.code.0 == "table_by_hand")
            .collect();
        let (text, _) = cpl_diag::splice_fixes(&src, &found);
        text[decl.len()..]
            .lines()
            .nth(2)
            .unwrap_or_default()
            .to_string()
    };
    assert_eq!(fixed("a[i] * 2"), "p = table([a[i] * 2 for i in 0..2])");
    assert_eq!(
        fixed("(if a[i] > 1: a[i] else: 0)"),
        "p = table([(if a[i] > 1: a[i] else: 0) for i in 0..2])"
    );
}

/// A hole that is a whole condition is written bare: `if n == 0:`, not
/// `if (n == 0):`. Rule: R06.4.9.
#[test]
fn a_hole_written_as_a_whole_condition_keeps_no_parens() {
    let decl = "@lint(elif_unwrap, from: \"if a:\\n    first\\nelif (self ?? c) > y:\\n    let w = self!\\n    body\", to: \"if a:\\n    first\\nelif let w = self, w > y:\\n    body\", holes: \"a, first, c, y, body\")\npub fn some_and(self: ?Int, p: Int -> Bool) -> Bool = if let Some(v) = self: p(v) else: false\n";
    let src = format!("{decl}let n = 2\nlet best: ?Int = Some(7)\nif n == 0:\n    print(0)\nelif (best ?? 0) > 4:\n    let b = best!\n    print(b)\n");
    let fixes: Vec<String> = crate::lint(&src, &crate::Config::default())
        .into_iter()
        .filter(|d| d.code.0 == "elif_unwrap")
        .map(|d| d.fixes[0].edits[0].1.clone())
        .collect();
    assert_eq!(
        fixes,
        ["if n == 0:\n    print(0)\nelif let b = best, b > 4:\n    print(b)"]
    );
}

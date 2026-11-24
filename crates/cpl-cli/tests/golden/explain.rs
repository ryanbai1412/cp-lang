//! `cpl explain`: codes, copies, storage and memory against the judge.

use super::*;

#[test]
fn explain_shows_a_code_and_rejects_an_unknown_one() {
    misused("explain", &["explain"], "`explain` takes a code or a file");
    misused(
        "explain",
        &["explain", "E7777"],
        "`E7777` is not a diagnostic or lint code",
    );
    let out = cpl(Path::new("."), &["explain", "E0302"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with("E0302: "),
        "{}",
        text(&out.stdout)
    );
    let unknown = cpl(Path::new("."), &["explain", "E7777"]);
    assert_eq!(unknown.status.code(), Some(EXIT_USAGE.into()));
    assert!(text(&unknown.stderr).contains("`E7777` is not a diagnostic or lint code"));
    let json = cpl(Path::new("."), &["explain", "--json"]);
    let json = cpl_json::parse(&text(&json.stdout)).expect("cpl explain --json is JSON");
    assert!(json
        .as_array()
        .unwrap()
        .iter()
        .any(|e| e.at(&["code"]).as_str() == Some("E0302")));
}

/// `cpl explain --json` lists each heap copy left in the C++, with where and
/// why it owns its value, as the text report does.
#[test]
fn explain_json_lists_each_copy_and_why() {
    let dir = scratch("explain_json_copies");
    std::fs::write(
        dir.join("c.cp"),
        "fn f(rows: List<List<Int>>) -> Int:\n    var own = rows[0]\n    own.push(5)\n    own.len\nprint(f([[1], [2]]))\n",
    )
    .unwrap();
    let out = cpl(&dir, &["explain", "c.cp", "--json"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let json = cpl_json::parse(&text(&out.stdout)).expect("JSON");
    let copy = json
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e.at(&["lowering"]).as_str() == Some("copy"))
        .unwrap_or_else(|| panic!("{}", text(&out.stdout)));
    assert_eq!(copy.at(&["site"]).as_str(), Some("2:15"));
    assert!(copy
        .at(&["detail"])
        .as_str()
        .unwrap()
        .contains("changed later"));
}

/// A file's explanation ends with the specializations its C++ takes and
/// the bound behind each, and each index with whether the checked builds
/// check it and, where they don't, why; `--json` has the same records.
#[test]
fn explain_shows_the_specializations_chosen_and_the_checks_left() {
    let dir = scratch("explain_chosen_checked");
    std::fs::write(
        dir.join("a.cp"),
        concat!(
            "let n: 1..=1000 = read()\n",
            "let xs: [0..n]Int = read()\n",
            "let i: Int = read()\n",
            "var k = [0, 0, 0, 0]\n",
            "for j in 0..3: k[j + 1] -= k[j]\n",
            "domain V = 0..n\n",
            "var seen: [V]Bool = false\n",
            "for v in V: seen[v] = true\n",
            "print(xs[i], k[0], yes(seen[V(0)]))\n",
        ),
    )
    .unwrap();
    std::fs::write(dir.join("plain.cp"), "print(1)\n").unwrap();
    let out = cpl(&dir, &["explain", "a.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    for line in [
        "  [chosen] domain V -> `int` positions: a plain domain holds at most i32::MAX\n",
        "  [no check] 5:16  k[j + 1] -> one check before its loop covers it\n",
        "  [no check] 8:13  seen[v] -> its index's type keeps it in range\n",
        "  [check] 9:7  xs[i]\n",
        "  [check] 9:14  k[0]\n",
    ] {
        assert!(stdout.contains(line), "{line}{stdout}");
    }
    assert!(!stdout.contains("Not shown yet"), "{stdout}");
    let plain = text(&cpl(&dir, &["explain", "plain.cp"]).stdout);
    assert!(
        plain.ends_with(
            "Specializations chosen for plain.cp: none\nChecks left per build for plain.cp: no index\n"
        ),
        "{plain}"
    );
    let json = cpl(&dir, &["explain", "a.cp", "--json"]);
    let json = cpl_json::parse(&text(&json.stdout)).expect("JSON");
    let rows = json.as_array().unwrap();
    let has = |site: &str, lowering: &str| {
        rows.iter().any(|e| {
            e.at(&["site"]).as_str() == Some(site) && e.at(&["lowering"]).as_str() == Some(lowering)
        })
    };
    assert!(has("9:7", "index_checked"), "{rows:?}");
    assert!(has("8:13", "index_unchecked"), "{rows:?}");
    assert!(has("", "specialization"), "{rows:?}");
}

/// Each `Int` `+ - *` says whether it keeps its overflow check, from the
/// same range facts that decide the C++.
#[test]
fn explain_shows_each_overflow_check_left() {
    let dir = scratch("explain_overflow_checked");
    std::fs::write(
        dir.join("a.cp"),
        "let n: 1..=1000 = read()\nlet a: Int = read()\nvar s = n * n\ns += a\nprint(s, a + 1)\n",
    )
    .unwrap();
    let out = cpl(&dir, &["explain", "a.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    let stdout = text(&out.stdout);
    for line in [
        "  [no check] 3:9  n * n -> its operands' proven ranges keep it in `Int`\n",
        "  [check] 4:1  s += a\n",
        "  [check] 5:10  a + 1\n",
    ] {
        assert!(stdout.contains(line), "{line}{stdout}");
    }
    let json = cpl(&dir, &["explain", "a.cp", "--json"]);
    let json = cpl_json::parse(&text(&json.stdout)).expect("JSON");
    let rows = json.as_array().unwrap();
    let has = |site: &str, lowering: &str| {
        rows.iter().any(|e| {
            e.at(&["site"]).as_str() == Some(site) && e.at(&["lowering"]).as_str() == Some(lowering)
        })
    };
    assert!(has("3:9", "overflow_unchecked"), "{rows:?}");
    assert!(has("5:10", "overflow_checked"), "{rows:?}");
}

/// The peak is reported against the memory limit of the file's judge
/// (spec/tools/judge-profiles.md, Rjudge.7).
#[test]
fn explain_reports_the_peak_against_the_judges_memory_limit() {
    let dir = scratch("explain_against_limit");
    let small = "[judge.small]\nextends = \"generic\"\nmemory_mb = 16\n";
    std::fs::write(dir.join("cp.toml"), small).unwrap();
    let src = "// judge: small\nvar a: [0..1000000]Int = 0\na[5] = 1\nprint(a[5])\n";
    std::fs::write(dir.join("m.cp"), src).unwrap();
    let out = text(&cpl(&dir, &["explain", "m.cp"]).stdout);
    assert!(out.contains("of 16 MB on small"), "{out}");
}

/// A `[V.ranges]T` table is a triangle, not a grid: explain counts its
/// n(n + 1)/2 cells and names the end-major order the C++ and the
/// interpreter share.
#[test]
fn explain_names_a_ranges_tables_layout() {
    let dir = scratch("explain_ranges");
    std::fs::write(
        dir.join("r.cp"),
        "domain I = 0..400\nvar cost: [I.ranges]Int = 0\nprint(cost[I.range])\n",
    )
    .unwrap();
    let out = text(&cpl(&dir, &["explain", "r.cp"]).stdout);
    assert!(
        out.contains(
            "8 B per element x 80200 = 641 KB, end-major triangle, n(n + 1)/2 cells: \
             lo..hi at (hi - 1) * hi / 2 + lo"
        ),
        "{out}"
    );
}

/// A file's explanation opens with each top-level value's bytes: per
/// element and at the input bounds for an array, the niche of an optional,
/// field offsets and padding, and the peak the fixed-size ones take; the
/// same numbers hover shows, also in `--json`.
#[test]
fn explain_a_file_shows_each_values_storage() {
    let dir = scratch("explain_storage");
    std::fs::write(
        dir.join("s.cp"),
        "domain V = 0..1000\nlet n: Int = read()\nlet a: [V](Int, 0..=9) = (0, 0)\n\
         let o: ?V = None\nlet xs: [n]Int = read()\nprint(n, o, a[0].0, xs.len)\n",
    )
    .unwrap();
    let out = cpl(&dir, &["explain", "s.cp"]);
    assert_eq!(out.status.code(), Some(0), "{}", text(&out.stderr));
    assert!(
        text(&out.stdout).starts_with(
            "Storage for s.cp (at the input bounds):\n\
             \x20 n   Int              8 B\n\
             \x20 a   [V](Int, 0..=9)  16 B per element x 1000 = 16 KB, fields @0 @8, 7 B padding\n\
             \x20 o   ?V               2 B, None is a spare value (no flag)\n\
             \x20 xs  [n]Int           8 B per element, no bound on the count\n\
             \x20 peak 16 KB of 256 MB on generic, not counting xs\n"
        ),
        "{}",
        text(&out.stdout)
    );
    let json = cpl_json::parse(&text(&cpl(&dir, &["explain", "s.cp", "--json"]).stdout)).unwrap();
    let rows = json.as_array().unwrap();
    let o = rows
        .iter()
        .find(|r| r.at(&["site"]).as_str() == Some("o"))
        .unwrap();
    assert_eq!(o.at(&["lowering"]).as_str(), Some("storage"));
    assert_eq!(
        o.at(&["detail"]).as_str(),
        Some("?V: 2 B, None is a spare value (no flag)")
    );
}

/// Explain's bytes per element are the C++'s: a `1..=10 ** 9` array read
/// into a generic std body is 4 B a cell in both, never widened to 8.
#[test]
fn explain_counts_a_narrow_array_at_the_cpp_width() {
    let dir = scratch("explain_narrow_generic");
    let src = "let n: 2..=600_000 = read()\nlet b: [0..n]1..=10 ** 9 = read()\n\
               fn f<A: Positional<Int>>(a: A) -> Int = a.nth(0)\n\
               var res = f(b)\nfor (x, y) in b.adjacent: res += x - y\nprint(res)\n";
    std::fs::write(dir.join("m.cp"), src).unwrap();
    let out = text(&cpl(&dir, &["explain", "m.cp"]).stdout);
    assert!(out.contains("4 B per element x 600000 = 2.4 MB"), "{out}");
    let built = cpl(&dir, &["build", "m.cp", "-o", "m.cpp"]);
    assert!(built.status.success(), "{}", text(&built.stderr));
    let cpp = std::fs::read_to_string(dir.join("m.cpp")).unwrap();
    assert!(cpp.contains("std::vector<uint32_t> b"), "{cpp}");
}

/// A struct on the heap lists its fields, its type arguments in place: a max-plus
/// `Mat<?Int>` shows the `raw List<Int>` its products and its build store, beside `cells`.
#[test]
fn explain_a_struct_on_the_heap_lists_its_fields() {
    let dir = scratch("explain_struct_fields");
    let src = "domain V = 0..3\nlet d: [V, V]?Int = [V, V] (i, j) => if i.ord <= j.ord: Some(i.ord) else: None\n\
               let m = Mat(d, over: Semiring(max, +))\nprint((m * m).get(0, 2) ?? -1)\n";
    std::fs::write(dir.join("m.cp"), src).unwrap();
    let out = text(&cpl(&dir, &["explain", "m.cp"]).stdout);
    let line = out
        .lines()
        .find(|l| l.trim_start().starts_with("m "))
        .unwrap_or("");
    assert!(
        line.contains("cells List<?Int>: grows on the heap; ?Int: 8 B each"),
        "{out}"
    );
    assert!(
        line.contains("raw List<Int>: grows on the heap; Int: 8 B each"),
        "{out}"
    );
}

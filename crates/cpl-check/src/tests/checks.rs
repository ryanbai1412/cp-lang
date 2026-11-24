//! One rule per test: the prelude and stdlib tables, `memo`, `items`, rows,
//! refused lends and recovery.

use super::*;

#[test]
fn a_generic_fn_takes_its_type_arguments_after_two_colons() {
    let head = "domain A = 0..3\nfn firsts<D>(k: Int) -> Int: k + D.len\n";
    assert!(codes(&format!("{head}print(firsts<A>(1), firsts::<A>(1))\n")).is_empty());
    let diag = one(&format!("{head}print(firsts < A)\n"));
    assert_eq!(
        diag.message,
        "`firsts` is a generic fn and is only called, as `firsts(...)`: its type parameters come from its arguments"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("to name one that no argument gives, write `firsts<T>(...)`")
    );
    let ok = format!("{head}print(firsts::<A>(1))\nlet k = 2\nprint(yes(k < 3))\n");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

#[test]
fn a_heap_starts_empty() {
    // `MinHeap` is the prelude's: the struct of `std/heap.cp`.
    let ok = "var h = MinHeap<(Int, Int)>()\nh.push((-3, 3))\nvar xs = List<Int>()\nxs.push(1)\n";
    assert!(codes(ok).is_empty(), "{:?}", check(ok));
}

/// dream/cf2183f reads its tree with no `use`: `Tree` is the prelude's,
/// as `Graph` and `Rooted` are.
#[test]
fn a_tree_reads_without_a_use() {
    let src = "domain V = 1..=5\nlet t = Rooted(Graph.undirected(read<Tree<V>>()), V.first)\nprint(t.size(V.first))\n";
    assert!(codes(src).is_empty(), "{:?}", check(src));
}

/// The tree dreams that the stdlib covers check clean, so a change that breaks
/// their spelling (`t.up(+, finish:)`, `t.virtual`, `t.below`) fails here.
#[test]
#[ignore = "corpus/problems/*/dream.cp are v0.2; ported with the corpus"]
fn the_tree_dreams_check() {
    for dream in ["cf2035f", "cf2124i", "cf2183d2", "cf2237i1"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../corpus/problems/{dream}/dream.cp"));
        let found = check(&std::fs::read_to_string(path).unwrap());
        assert!(found.is_empty(), "{dream}: {found:?}");
    }
}

#[test]
fn reading_an_enum_needs_its_tags() {
    let rest = "let n: 1..=3 = read()\ndomain Row = 1..=n\nlet g: [Row, Row]Tile = read()\n";
    let diag = one(&format!("enum Tile:\n    X\n    O\n{rest}"));
    assert_eq!(
        diag.message,
        "reading a `Tile` needs to know how its variants are written in the input"
    );
    assert_eq!(
        diag.help.as_deref(),
        Some("put `@tags(\"X\", \"O\")` above the enum")
    );
    let ok = format!("@tags(\"X\", \"O\")\nenum Tile:\n    X\n    O\n{rest}");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

/// A missing stdlib `use` goes into the file's line for that module, else
/// below its other `use`s, else above its first line of code.
#[test]
fn an_import_joins_its_module_line_or_the_file_header() {
    let cases = [
        (
            "use std.math.{gcd}\nprint(1)\n",
            "use std.math.{gcd, lcm}\nprint(1)\n",
        ),
        (
            "use std.math.{}\nprint(1)\n",
            "use std.math.{lcm}\nprint(1)\n",
        ),
        (
            "use std.seq.{zip}\nuse std.dsu.{Dsu}\nprint(1)\n",
            "use std.seq.{zip}\nuse std.dsu.{Dsu}\nuse std.math.{lcm}\nprint(1)\n",
        ),
        (
            "// a header\n\nprint(1)\n",
            "// a header\n\nuse std.math.{lcm}\nprint(1)\n",
        ),
        (
            "use std.seq.{zip}",
            "use std.seq.{zip}\nuse std.math.{lcm}\n",
        ),
    ];
    for (text, want) in cases {
        let (span, edit) = crate::import_edit(text, "math", "lcm");
        let mut got = text.to_string();
        got.replace_range(span.start as usize..span.end as usize, &edit);
        assert_eq!(got, want);
    }
    // The checker's quick fix uses it, wherever in the file the name is.
    let src = "use std.math.{gcd}\nlet a = gcd(4, 6)\nprint(floor_sum(a, 3, 1, 0))\n";
    let fix = &one(src).fixes[0];
    assert_eq!(
        fix.edits,
        vec![(Span::point(17), ", floor_sum".to_string())]
    );
}

/// `STD_NAMES` is what the hint for a missing `use` is read from. It is
/// built from `std/` itself, sorted for the binary search, without
/// the names that more than one module declares.
#[test]
fn the_std_name_table_is_read_off_the_standard_library() {
    let names = crate::prelude::std_names();
    assert!(names.windows(2).all(|pair| pair[0].0 < pair[1].0));
    assert_eq!(crate::prelude::std_module("Fenwick"), Some("fenwick"));
    assert_eq!(crate::prelude::std_module("no_such_name"), None);
    assert_eq!(crate::prelude::std_module("runs"), Some("seq"));
}

/// The compiler carries `std/` without its tests, line for line, so
/// a span into a bundled module still points at the right line.
#[test]
fn the_bundled_stdlib_drops_tests_and_keeps_every_line() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../std");
    let modules = crate::modules::std_modules();
    assert!(modules.iter().any(|(name, _)| *name == "seq"));
    for (name, bundled) in modules {
        let file = std::fs::read_to_string(root.join(format!("{name}.cp"))).expect("a module file");
        assert!(!bundled.contains("@test"), "{name} keeps a test");
        assert_eq!(bundled.lines().count(), file.lines().count(), "{name}");
        for (kept, line) in bundled.lines().zip(file.lines()) {
            assert!(kept.is_empty() || kept == line, "{name}: {kept}");
        }
    }
    let seq = crate::modules::bundled_std(&["seq".to_string()]).expect("seq is bundled");
    assert!(seq.contains("pub fn bsearch<D>"));
}

/// `std/README.md` is the design of the whole surface, so a public
/// name of `std/` it never mentions is a name nobody designed.
#[test]
#[ignore = "std/README.md does not name every public std name yet"]
fn the_stdlib_design_names_every_public_name() {
    let design = include_str!("../../../../std/README.md");
    let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    let words: std::collections::HashSet<&str> = design.split(|c: char| !word(c)).collect();
    let missing: Vec<&str> = crate::prelude::std_names()
        .iter()
        .map(|(name, _)| *name)
        .filter(|name| !words.contains(name))
        .collect();
    assert!(
        missing.is_empty(),
        "std/README.md does not mention {missing:?}"
    );
}

/// `@memo` on a fn that neither back end keeps a table for is an error,
/// not a fn that quietly runs every time.
#[test]
fn memo_on_a_fn_it_cannot_remember_is_reported() {
    let memo = |src: &str| {
        let d = one(src);
        assert_eq!(d.code.0, "E0311", "{d:?}");
        // The decorator is what to change.
        assert_eq!(
            &src[d.span.start as usize..d.span.end as usize],
            "@memo",
            "{d:?}"
        );
        assert!(d.help.is_some(), "{d:?}");
        d.message
    };
    assert_eq!(
        memo("@memo\nfn take(mut xs: List<Int>, i: Int) -> Int:\n    xs.push(i)\n    i\n"),
        "`@memo` can't remember `take`: it changes its argument `xs`"
    );
    assert_eq!(
        memo("struct S:\n    k: Int\n    @memo\n    fn get(self, i: Int) -> Int: self.k + i\n"),
        "`@memo` can't remember `get`: it is a fn of `S`"
    );
    assert_eq!(
        memo("@memo\nfn upto(n: Int) -> yields Int:\n    for i in 0..n: yield i\n"),
        "`@memo` can't remember `upto`: it yields its values one by one"
    );
    assert_eq!(
        memo("@memo\nfn same<T>(x: T) -> T: x\n"),
        "`@memo` can't remember the generic fn `same`"
    );
    // No value: written `-> ()`, a body that only acts, a tail `print`.
    for body in [
        "fn show(x: Int) -> ():\n    print(x)\n",
        "fn show(x: Int):\n    var t = x\n    t += 1\n",
        "fn show(x: Int):\n    print(x)\n",
    ] {
        assert_eq!(
            memo(&format!("@memo\n{body}")),
            "`@memo` has nothing to remember: `show` gives no value"
        );
    }
    // A fn inside a fn is held to the same.
    assert_eq!(
        memo("fn outer() -> Int:\n    @memo\n    fn inner(mut ys: List<Int>) -> Int: ys.len\n    var ys: List<Int> = []\n    inner(mut ys)\n"),
        "`@memo` can't remember `inner`: it changes its argument `ys`"
    );
}

#[test]
fn memo_on_a_fn_with_a_value_and_plain_arguments_is_fine() {
    for src in [
        "@memo\nfn fib(n: Int) -> Int:\n    if n < 2: return n\n    fib(n - 1) + fib(n - 2)\nprint(fib(50))\n",
        // The return type is read off the body.
        "@memo\nfn twice(n: Int):\n    if n > 3: return 5\n    n * 2\nprint(twice(4))\n",
        "@memo\nfn sq(x: Int) -> Int = x * x\nprint(sq(3))\n",
        "let n: 1..=20 = read()\ndomain V = 0..n\n@memo(into: dp)\nfn ways(v: V) -> Int:\n    if let u = v + 1: return ways(u) + 1\n    1\nprint(ways(V.first!))\n",
        "fn outer(k: Int) -> Int:\n    @memo\n    fn inner(i: Int) -> Int:\n        if i == 0: return k\n        inner(i - 1) + 1\n    inner(5)\nprint(outer(2))\n",
    ] {
        assert_eq!(check(src), Vec::new(), "{src}");
    }
}

/// `adj.out(u)` reads a table of neighbour lists. On any other table the
/// checker says so, as `cpl run` and `cpl build` do.
#[test]
fn out_on_a_table_that_holds_no_lists_is_reported() {
    let rest = "let n = 3\ndomain V = 0..n\n";
    let d = one(&format!(
        "{rest}let a: [V]Int = [V] v => 0\nprint(a.out(V(1)))\n"
    ));
    assert_eq!(d.code.0, "E0203", "{d:?}");
    assert_eq!(d.help.as_deref(), Some("index it: `a[v]`"));
    let ok = format!("{rest}var adj: [V]List<V> = [V] v => []\nprint(adj.out(V(0)).len)\n");
    assert!(codes(&ok).is_empty(), "{:?}", check(&ok));
}

/// The branch an `if const` does not pick is not compiled, so it is not
/// checked; one whose test waits for a type argument still is.
#[test]
fn the_untaken_branch_of_an_if_const_is_not_checked() {
    let skipped =
        "const FAST = false\nif const FAST:\n    print(no_such_name)\nelse:\n    print(1)\n";
    assert!(codes(skipped).is_empty(), "{:?}", check(skipped));
    let picked = "if const 1 < 2:\n    print(1)\nelse:\n    print(no_such_name)\n";
    assert!(codes(picked).is_empty(), "{:?}", check(picked));
    let taken = "const FAST = true\nif const FAST:\n    print(no_such_name)\n";
    assert_eq!(codes(taken).len(), 1, "{:?}", check(taken));
}

/// A test for `none` leaves the name holding a value where the test says
/// so, and nowhere the name is assigned again.
#[test]
fn a_tested_optional_reads_as_its_value() {
    let head = "fn f(x: ?Int) -> Int:\n";
    for body in [
        "    if x == None: return 0\n    x + 1\n",
        "    if x != None: return x + 1\n    0\n",
        "    if x == None or x < 0: return 0\n    x\n",
        "    if x != None: return x! + 1\n    x ?? 0\n",
    ] {
        let src = format!("{head}{body}");
        assert!(codes(&src).is_empty(), "{src}\n{:?}", check(&src));
    }
    // Assigned in the code the test covers: still an optional there.
    let src = "fn g(x: ?Int) -> Int:\n    var y = x\n    if y == None: return 0\n    y = None\n    y + 1\n";
    assert!(!codes(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_match_on_plain_variants_needs_an_arm_for_each() {
    let with = |arms: &str| format!("enum C: R | G | B\nfn f(c: C) -> Int:\n    match c:\n{arms}");
    let d = one(&with("        R: 1\n        G: 2\n"));
    assert_eq!(d.message, "this `match` has no arm for `B`");
    let d = one(&with("        R if true: 1\n        G | B: 2\n"));
    assert_eq!(d.message, "this `match` has no arm for `R`");
    for arms in [
        "        R: 1\n        G | B: 2\n",
        "        R: 1\n        _: 2\n",
        "        R: 1\n        other: 2\n",
    ] {
        assert!(check(&with(arms)).is_empty(), "{:?}", check(&with(arms)));
    }
}

#[test]
fn a_getter_reads_the_same_with_empty_parentheses() {
    let src = "struct P(x: Int):\n    pub @get fn twice(self) -> Int = 2 * x\nfn f(p: P) -> Int = p.twice() + p.twice\n";
    assert!(check(src).is_empty(), "{:?}", check(src));
}

#[test]
fn a_method_that_calls_a_mut_self_method_takes_mut_self() {
    let src = |receiver: &str| {
        format!(
            "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\n    fn peek({receiver}) -> Int:\n        self.bump()\n        n\n"
        )
    };
    assert_eq!(check(&src("mut self")), vec![]);
    assert_eq!(
        messages(&check(&src("self"))),
        ["this method takes `self` without `mut`, so `self` can't be changed by `bump`"]
    );
    // A field of `self` is part of it.
    let nested = "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\nstruct Pair:\n    a: C\n    fn peek(self) -> Int:\n        a.bump()\n        a.n\n";
    assert_eq!(
        messages(&check(nested)),
        ["this method takes `self` without `mut`, so `self` can't be changed by `bump`"]
    );
}

#[test]
fn a_range_written_as_a_structs_type_argument_is_an_anonymous_domain() {
    assert_eq!(check("let n = 4\nvar d = Dsu<0..n>()\n"), vec![]);
}

#[test]
fn a_mut_self_method_needs_a_receiver_that_can_change() {
    let decl = "struct C:\n    n: Int\n    fn bump(mut self):\n        n += 1\n";
    assert_eq!(check(&format!("{decl}var c = C(0)\nc.bump()\n")), vec![]);
    assert_eq!(
        check(&format!("{decl}fn f(mut c: C):\n    c.bump()\n")),
        vec![]
    );
    assert_eq!(
        messages(&check(&format!("{decl}let c = C(0)\nc.bump()\n"))),
        ["`c` is a `let`, so it can't be changed by `bump`"]
    );
    assert_eq!(
        messages(&check(&format!(
            "{decl}fn f():\n    let c = C(0)\n    c.bump()\n"
        ))),
        ["`c` is a `let`, so it can't be changed by `bump`"]
    );
    assert_eq!(
        messages(&check(&format!("{decl}fn f(c: C):\n    c.bump()\n"))),
        ["`c` is a parameter without `mut`, so it can't be changed by `bump`"]
    );
}

#[test]
fn items_gives_each_element_with_its_index() {
    let src = "domain I = 0..3\nlet a: [I]Int = [4, 5, 6]\nlet xs = [true, false]\n";
    // A position of `I` for `[I]T`, an `Int` for a list.
    assert_eq!(
        codes(&format!("{src}for (i, x) in a.items: print(a[i] + x)\n")),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in a.items:\n    let j: Int = i\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in a.items:\n    let s: Str = x\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (i, x) in xs.items:\n    let n: Int = x\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}let kept = xs.items\nlet k: Int = kept[0].0\n"
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!(
            "{src}let kept = xs.items\nlet k: Int = kept[0].1\n"
        )),
        ["E0302"]
    );
}

#[test]
fn items_of_a_str_a_map_and_a_grid() {
    let src = concat!(
        "domain R = 0..2\n",
        "var g: [R, 0..3]Int = 0\n",
        "var m = Map<Str, Int>()\n",
        "let s = \"abc\"\n",
    );
    let ok = concat!(
        "for (i, c) in s.items:\n    let n: Int = i\n    let d: Char = c\n",
        "for (k, v) in m.items:\n    let key: Str = k\n    let n: Int = v\n",
        "for ((r, c), x) in g.items:\n    let row: R = r\n    let n: Int = c + x\n",
    );
    assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new());
    assert_eq!(
        codes(&format!(
            "{src}for (i, c) in s.items:\n    let n: Int = c\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for (k, v) in m.items:\n    let n: Int = k\n"
        )),
        ["E0302"]
    );
    assert_eq!(
        codes(&format!(
            "{src}for ((r, c), x) in g.items:\n    let n: Int = r\n"
        )),
        ["E0302"]
    );
}

#[test]
fn a_row_takes_the_larger_cells_of_a_row_of_its_own_type() {
    let src = concat!(
        "domain V = 0..3\ndomain W = 0..3\n",
        "var a: [V]Int = 0\nvar b: [V]Int = 1\nvar w: [W]Int = 0\nvar s: [V]Str = \"\"\n",
        "var xs = [1, 2]\nvar best = 0\n",
    );
    for ok in [
        "a max= b\n",
        "if a min= b: print(a)\n",
        "xs max= [3, 0]\n",
        "best max= 4\n",
        "print(yes(best min= 2))\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
    for bad in [
        "a max= w\n",
        "print(yes(a min= s))\n",
        "a max= 3\n",
        "let hit = xs max= [\"a\"]\n",
    ] {
        assert_eq!(codes(&format!("{src}{bad}")), ["E0302"], "{bad}");
    }
}

#[test]
fn a_filled_list_holds_the_element_type_asked_for() {
    let src =
        "type Z = Mod<998244353>\nfn zeros(k: Int) -> List<Z>:\n    return List.filled(k, 0)\n";
    assert_eq!(
        codes(&format!(
            "{src}let a: List<Z> = List.filled(3, 0)\nlet z: Z = a[0]\n"
        )),
        Vec::<&str>::new()
    );
    assert_eq!(
        codes(&format!("{src}let a: List<Z> = List.filled(3, \"x\")\n")),
        ["E0302"]
    );
}

#[test]
fn a_row_of_a_grid_takes_the_larger_cells_of_another_row() {
    let src = concat!(
        "domain K = 0..3\n",
        "var g: [K, 0..4]Int = 0\nvar s: [K, 0..4]Str = \"\"\nvar row: [0..4]Int = 0\n",
        "let (k, v) = (K(0), K(2))\n",
    );
    for ok in [
        "g[k, ..] max= g[v, ..]\n",
        "if g[k, ..] min= g[v, ..]: print(g[k, ..])\n",
        "g[.., 1] max= g[.., 2]\n",
        "g[k, 1..3] min= g[v, 0..2]\n",
        "g[k, ..] max= row\n",
        "g[k, ..] max= [1, 2, 3, 4]\n",
        // One value goes against every cell of the part.
        "g[k, ..] max= 3\n",
        "g[k, 1..3] += 2\n",
        "g[.., 1] min= row[0]\n",
        "row[1..3] -= 1\nrow[..] max= 0\n",
        "if row[..] min= 2: print(row)\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
    for bad in [
        "g[k, ..] max= s[v, ..]\n",
        "print(yes(g[k, ..] min= s[v, ..]))\n",
        "g[k, ..] max= \"x\"\n",
        "row[..] min= 2.5\n",
    ] {
        assert_eq!(codes(&format!("{src}{bad}")), ["E0302"], "{bad}");
    }
}

#[test]
fn a_list_written_out_beside_a_plus_holds_the_element_type_asked_for() {
    let src = "type Z = Mod<998244353>\nlet rest: List<Z> = [Z(2), Z(3)]\nfn pad(a: List<Z>) -> List<Z>:\n    return [0] + a + [0]\n";
    for ok in [
        "let xs: List<Z> = [0] + rest\nlet z: Z = xs[0]\n",
        "let xs: List<Z> = rest + [0, 1]\n",
        "let xs: List<Z> = [1] + [2] + rest\n",
        "let ys: List<F64> = [1] + [2.5]\n",
    ] {
        assert_eq!(codes(&format!("{src}{ok}")), Vec::<&str>::new(), "{ok}");
    }
}

/// `xs.sort(by:)` sorts where the list is: a key that reaches the list,
/// by name or through a fn, would read or grow it mid-sort (a
/// use-after-free in the C++). `sorted(by:)` works on a copy.
#[test]
fn a_sort_key_that_reaches_its_own_list_is_refused() {
    const FNS: &str = "var xs = [3, 1, 2]\nvar n = 0\nfn grow(x: Int) -> Int:\n    xs.push(1)\n    x\nfn peek(x: Int) -> Int = x + xs.len\nfn count(x: Int) -> Int:\n    n += 1\n    x\n";
    for call in [
        "xs.sort(by: grow)\n",
        "xs.sort(by: peek)\n",
        "xs.sort(by: x => xs[0] - x)\n",
        "xs.sort(by: x => grow(x))\n",
    ] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0602"],
            "{call}"
        );
    }
    for call in [
        "xs.sort(by: count)\n",
        "xs.sort(by: x => -x)\n",
        "let ys = xs.sorted(by: grow)\n",
        "var ys = [1]\nys.sort(by: peek)\n",
    ] {
        assert_eq!(check(&format!("{FNS}{call}")), vec![], "{call}");
    }
}

/// `cpl run` no longer stops a call that touches a variable it was lent
/// `mut`, so the check has to: one variable twice in a call is refused
/// here, while a fn value reading it during the call is not.
#[test]
fn one_variable_lent_twice_to_a_call_is_refused() {
    const FNS: &str = "fn two(mut x: List<Int>, mut y: List<Int>): x.push(y.pop()!)\nfn one(mut x: List<Int>, y: List<Int>): x.push(y.len)\nvar xs = [1]\n";
    for call in ["two(mut xs, mut xs)\n", "one(mut xs, xs)\n"] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0601"],
            "{call}"
        );
    }
    assert_eq!(
        check(&format!(
            "{FNS}var ys = [2]\ntwo(mut xs, mut ys)\none(mut xs, ys)\n"
        )),
        vec![]
    );
    let closure = "domain P = 0..3\nvar m = Map<Str, P>()\nm.update(\"a\", P(0), v => P(m.len - 1))\nprint(m[\"a\"])\n";
    assert_eq!(check(closure), vec![]);
}

#[test]
fn a_try_is_refused_where_it_is_not_always_run() {
    for source in [
        "fn f(c: Bool, x: ?Int) -> ?Bool:\n    Some(c and x? > 0)\n",
        "fn f(c: Bool, x: ?Int) -> ?Bool:\n    Some(c or 1 + x? > 0)\n",
        "fn f(p: ?Int, x: ?Int) -> ?Int:\n    Some(p ?? (1 + x?))\n",
    ] {
        let found = codes(source);
        assert_eq!(found, ["E0309"], "{source}");
    }
    assert_eq!(
        codes("fn f(n: Int, x: ?Int) -> ?Int:\n    var i = 0\n    while i + x? < n: i += 1\n    Some(i)\n"),
        ["E0803"]
    );
}

/// `debug(..)` is left out of the C++ a judge runs, so what it is given
/// must change nothing: a `pop`, a `read` or a `mut` call inside would
/// happen in `cpl run` and not on the judge.
#[test]
fn debug_of_something_that_changes_the_program_is_refused() {
    const FNS: &str = "var q = [1, 2, 3]\nvar total = 0\nvar y = 4\nfn add(k: Int) -> Int:\n    total += k\n    total\nfn shout(k: Int) -> Int:\n    print(k)\n    k\nfn inc(mut x: Int) -> Int:\n    x += 1\n    x\nfn twice(k: Int) -> Int = k * 2\n";
    for call in [
        "debug(q.pop())\n",
        "debug(add(2))\n",
        "debug(shout(1))\n",
        "debug(inc(mut y))\n",
        "debug(q.len, twice(add(1)))\n",
        "let n: Int = 0\ndebug(read() + n)\n",
    ] {
        let diags = check(&format!("{FNS}{call}"));
        assert_eq!(
            diags.iter().map(|d| d.code.0).collect::<Vec<_>>(),
            vec!["E0312"],
            "{call}"
        );
        assert!(
            diags[0]
                .help
                .as_deref()
                .is_some_and(|help| help.contains("debug(x)")),
            "{call}"
        );
    }
    for call in [
        "debug(q, q.len, twice(3), q.sorted(), q.max())\n",
        "let x = q.pop()\ndebug(x)\n",
        "debug(twice(y) + 1)\n",
    ] {
        assert_eq!(check(&format!("{FNS}{call}")), vec![], "{call}");
    }
}

#[test]
fn a_dropped_closer_inside_a_call_or_index_is_the_only_error() {
    // `f(a[i, j)` closes the call, so the call or index sees one argument
    // too many or too few; that is the missing `]`, not a second error.
    for (port, good, bad) in [
        ("cf2096b", "max(l[i], r[i]) for", "max(l[i, r[i]) for"),
        (
            "cf2096h",
            "(even_prod[a], odd_prod[a], even_prod[b], odd_prod[b]) =",
            "(even_prod[a, odd_prod[a], even_prod[b], odd_prod[b]) =",
        ),
    ] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../../corpus/problems/{port}/port.cp"));
        let src = std::fs::read_to_string(path).unwrap();
        assert!(src.contains(good), "{port}: {good}");
        assert_eq!(
            codes(&src.replacen(good, bad, 1)),
            ["E0101"],
            "{port}: {bad}"
        );
    }
}

#[test]
fn an_unreadable_character_is_reported_where_it_is() {
    // The lexer poisons the character's span so nothing else is reported
    // there; its own diagnostic must not fall to that poison.
    let src = "let $x = 1\nprint(x)\n";
    let diags = check(src);
    assert_eq!(diags[0].code.0, "E0004", "{diags:#?}");
    assert_eq!(diags[0].span, Span::new(4, 5));
}
#[test]
fn fields_loops_check_each_field_and_refuse_what_has_none() {
    let src =
        "struct P(x: Int, s: Str)\nlet p = P(1, \"a\")\nfor f in fields(p):\n    print(f + 1)\n";
    let d = one(src);
    assert_eq!(d.code.0, "E0309", "{d:#?}");
    let d = one("for f in fields(3):\n    print(f)\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        (
            "E0302",
            "`fields` walks the fields of a struct or a tuple, not of a `Int`"
        )
    );
    let d = one("for f in fields((1, 2)):\n    if f > 1:\n        break\n");
    assert_eq!(d.code.0, "E0501");
    let d = one("for (a, b) in fields((1, 2), (1, 2, 3)):\n    print(a)\n");
    assert_eq!(d.code.0, "E0302");
    assert!(
        codes("fn fields(x: Int) -> List<Int> = [x]\nfor f in fields(3):\n    print(f)\n")
            .is_empty()
    );
}

#[test]
fn names_from_a_declaration_that_failed_to_parse_stay_bound() {
    // Uses of `l`, `b` and `f` are not "not in scope" and say nothing about
    // types: the one error is the one root cause.
    let src = "let ($l, b) = (4, 6)\nprint(l + b)\nfn $f(a: Int) -> Int:\n    return a + 1\nprint(f(2) + l)\n";
    assert_eq!(codes(src), ["E0004", "E0004"], "{:#?}", check(src));
}

#[test]
fn a_recovered_name_still_reports_errors_unrelated_to_it() {
    let src = "let ($l, b) = (4, 6)\nprint(l + b)\nprint(missing)\n";
    assert_eq!(codes(src), ["E0004", "E0201"], "{:#?}", check(src));
}
#[test]
fn a_fields_loop_over_an_enum_checks_each_variant_payload() {
    let head = "enum Op:\n    Add(Int, Int)\n    Name(Str)\n    Stop\n";
    assert!(codes(&format!("{head}for f in fields(Op.Stop):\n    print(f)\n")).is_empty());
    let d = one(&format!(
        "{head}for f in fields(Op.Stop):\n    print(f + 1)\n"
    ));
    assert_eq!(d.code.0, "E0309", "{d:#?}");
}

#[test]
fn a_fields_constructor_gives_each_field_a_value_of_its_type() {
    let head = "struct P(a: Int, b: Str)\nstruct Q(a: Int)\nlet p = P(1, \"x\")\n";
    assert!(codes(&format!(
        "{head}let a = P(read() for _ in fields(P))\nlet b = P(f for f in fields(p))\n"
    ))
    .is_empty());
    let d = one(&format!("{head}let b = P(1 for _ in fields(P))\n"));
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "the field `b` is a Str, but `1` is an Int")
    );
    let d = one(&format!("{head}let c = Q(f for f in fields(p))\n"));
    assert_eq!(
        d.message,
        "`fields` here walks values of the type it builds, `Q`; this is a `P`"
    );
    let d = one(&format!("{head}let c = P(f for f in fields(P))\n"));
    assert_eq!(
        d.help.as_deref(),
        Some("write `_`, as in `P(read() for _ in fields(P))`")
    );
}

#[test]
fn a_lazy_parameter_takes_the_expression_its_lambda_gives_back() {
    let head = "fn or_else<T>(a: ?T, lazy d: () -> T) -> T:\n    if let v = a:\n        return v\n    d()\n";
    assert!(codes(&format!(
        "{head}let x: ?Int = 3\nprint(or_else(x, 4), or_else(x, d: 5), or_else(x, () => 6))\n"
    ))
    .is_empty());
    let d = one(&format!(
        "{head}let x: ?Int = 3\nprint(or_else(x, \"a\"))\n"
    ));
    assert_eq!(d.code.0, "E0302", "{d:#?}");
    let d = one("fn f(lazy d: Int) -> Int = 0\n");
    assert_eq!(
        (d.code.0, d.help.as_deref()),
        ("E0302", Some("write `lazy d: () -> Int`"))
    );
    assert!(codes("let lazy = 3\nfn g(lazy: Int) -> Int = lazy\nprint(g(lazy))\n").is_empty());
}

#[test]
fn a_variadic_parameter_is_a_tuple_of_its_arguments() {
    assert!(codes(
        "fn count<Ts>(..xs: Ts) -> Int:\n    var n = 0\n    for _ in fields(xs):\n        n += 1\n    n\nprint(count(), count(1, \"a\", 2.5))\n"
    )
    .is_empty());
    assert!(
        codes("fn tag<Ts>(t: Str, ..xs: Ts):\n    print(t)\ntag(\"a\", 1, 2)\ntag(\"b\")\n")
            .is_empty()
    );
    let d = one("fn total(..xs: Int) -> Int = 0\nprint(total(1, 2))\n");
    assert_eq!(
        (d.code.0, d.message.as_str()),
        ("E0302", "`..xs` collects its arguments into a tuple, so its type is a type parameter, not `Int`")
    );
}

#[test]
fn self_in_trait_range_bounds_and_generic_optional_impls_check() {
    let trait_src = "\
trait Index:
    fn count() -> Int
    fn ord(self) -> 0..Self.count
    fn from_ord(i: 0..Self.count) -> Self
";
    assert!(codes(trait_src).is_empty(), "{:#?}", check(trait_src));

    let impl_src = "\
impl<T: Add> Add for ?T:
    fn add(a, b) -> ?T = if let x = a, let y = b: x + y else: None
impl<T: Eq> Eq for ?T
";
    assert_eq!(codes(impl_src), ["E0501", "E0501"]);

    let assoc_src = "\
trait Unwrap:
    type Out
    fn unwrap(self) -> Out
";
    assert!(codes(assoc_src).is_empty(), "{:#?}", check(assoc_src));
}

#[test]
fn fresh_domain_escapes_are_rejected_with_e0313() {
    let cases = [
        // Returning a local fresh domain without an existential return type.
        "fn bad(k: Int) -> [0..k]Int:\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    a\n",
        // Returning a value of a local fresh domain from a function.
        "fn bad(k: Int):\n    domain Local = 0..k\n    return Local(0)\n",
        // Assigning a fresh-domain value to an outer variable.
        "var saved: Int = 0\nif true:\n    domain Inner = 0..5\n    let v = Inner(0)\n    var outer: ?Inner = None\n    outer = v\n",
        // Storing a fresh-domain value into an outer list.
        "var bag = []\nif true:\n    domain Inner = 0..5\n    bag.push(Inner(0))\n",
        // Capturing a fresh-domain value in a returned closure.
        "fn make(k: Int) -> () -> Int:\n    domain Local = 0..k\n    let a: [Local]Int = 0\n    () => a[Local(0)]\n",
    ];
    for (idx, src) in cases.iter().enumerate() {
        let diags = check(src);
        let codes: Vec<&str> = diags.iter().map(|d| d.code.0).collect();
        if idx == 2 {
            // `outer` is declared inside the same `if` block as `Inner`; no escape.
            assert!(!codes.contains(&"E0313"), "case {idx}: {diags:#?}");
        } else {
            assert!(
                codes.contains(&"E0313"),
                "case {idx} expected E0313, got {diags:#?}"
            );
        }
    }
}

#[test]
fn existential_domain_returns_and_runtime_read_fields_typecheck() {
    let src = "\
fn make_seq(k: Int) -> (domain P, [P]Int):
    domain Local = 0..k
    let a: [Local]Int = 0
    (Local, a)

fn make_pair(a: Int, b: Int) -> (domain A, domain B, [A]Int, [B]Int):
    domain X = 0..a
    domain Y = 0..b
    (X, Y, [X] i => i.ord, [Y] j => j.ord)

domain V = read(1..=100)
let m: 0..=100 = read()
let edges: [0..m](u: V, v: V, w: 0..=m) = read()
let (domain Q, vals) = make_seq(3)
let (domain A, domain B, xs, ys) = make_pair(2, 4)
print(edges.len, Q.count, vals[Q(0)], A.count, B.count, xs[A(0)], ys[B(0)])
";
    assert_eq!(check(src), vec![]);
}

#[test]
fn sym_storage_and_domain_iteration_typecheck() {
    let ok = "\
domain V = 1..=5
var adj: [Sym<V>]Bool = false
for (i, j) in V.pairs:
    adj[i, j] = true
    let p = (j, i)
    if adj[p]: print(i, j)
for i in V:
    for j in V.after(i): print(i, j)
    for j in V.before(i): print(j, i)
";
    assert_eq!(check(ok), vec![]);

    let bad = "\
domain V = 1..=5
domain U = 1..=5
var adj: [Sym<V>]Bool = false
let _ = adj[V.first!, U.first!]
";
    assert_eq!(codes(bad), vec![cpl_diag::codes::TYPE_MISMATCH.0]);
}
#[test]
fn typed_lambda_parameter_annotations_are_checked() {
    let ok = "\
fn apply(x: Int, f: fn(Int) -> Int) -> Int = f(x)
print(apply(5, (a: Int) => a + 1))
";
    assert_eq!(check(ok), vec![]);

    let bad_param = "\
fn apply(x: Int, f: fn(Int) -> Int) -> Int = f(x)
print(apply(5, (a: Str) => 1))
";
    assert_eq!(codes(bad_param), vec![cpl_diag::codes::TYPE_MISMATCH.0]);

    let bad_body = "let f = (a: Str) => a + 1\n";
    assert_eq!(codes(bad_body), vec![cpl_diag::codes::BAD_OPERANDS.0]);

    let bad_ty = "let f = (a: Nope) => a\n";
    assert_eq!(codes(bad_ty), vec![cpl_diag::codes::UNKNOWN_TYPE.0]);
}

#[test]
fn a_field_named_without_self_says_where_it_is() {
    let diags = check(
        "struct P(z: Int, lnds: Int):\n    fn up(self) -> Int = lnds + 1\n    fn fresh() -> Int = lnds\nprint(P(1, 2).up())\n",
    );
    assert_eq!(messages(&diags), ["no `lnds` in scope"]);
    assert_eq!(
        diags[0].help.as_deref(),
        Some("`lnds` is a field of `P`, and this method takes no `self`: add `self` as its first parameter")
    );
}

#[test]
fn an_unproven_literal_view_key_offers_the_position_and_the_source_key() {
    let src = "let a = [1, 1, 2, 3]\nfor r in a.runs(by: x => x):\n    print(r[0])\n";
    let diag = one(src);
    assert_eq!(diag.code.0, "E0305");
    let fixed: Vec<String> = diag
        .fixes
        .iter()
        .map(|fix| {
            let mut out = src.to_string();
            for (span, edit) in fix.edits.iter().rev() {
                out.replace_range(span.start as usize..span.end as usize, edit);
            }
            out
        })
        .collect();
    let line =
        |e: &str| format!("let a = [1, 1, 2, 3]\nfor r in a.runs(by: x => x):\n    print({e})\n");
    assert_eq!(fixed, [line("r.nth(0)"), line("a[0]")]);
}

/// L0610 waits for a body that is still `todo`: what it will change isn't
/// written yet.
#[test]
fn an_unwritten_body_does_not_flag_its_mut_params() {
    let call = "var a = [1, 2]\nprint(f(mut a))\n";
    let todo = format!("fn f(mut xs: List<Int>) -> Int = todo\n{call}");
    assert!(!codes(&todo).contains(&"L0610"), "{:?}", codes(&todo));
    let block = format!("fn f(mut xs: List<Int>) -> Int:\n    let n = xs.len\n    todo\n{call}");
    assert!(!codes(&block).contains(&"L0610"), "{:?}", codes(&block));
    let read = format!("fn f(mut xs: List<Int>) -> Int = xs.len\n{call}");
    assert!(codes(&read).contains(&"L0610"), "{:?}", codes(&read));
}

#[test]
fn an_arm_after_a_lowercase_binding_is_unreachable_and_names_the_variant() {
    let d = crate::tests::one(
        "let x = match cmp(2, 1):\n    less => 1\n    greater => 2\n    equal => 3\nprint(x)\n",
    );
    assert_eq!(d.code.0, "E0318");
    assert!(
        d.help.as_deref().unwrap().contains("did you mean `Less`?"),
        "{d:?}"
    );
    assert_eq!(d.fixes[0].edits[0].1, "Less");
    let ok = "let x = match 3:\n    3 => 1\n    n if n > 5 => 2\n    _ => 0\nprint(x)\n";
    assert!(crate::tests::check(ok).is_empty());
}

//! Optionals packed into one scalar or a spare value of what they hold.

use super::{cpp, decl, table};

#[test]
fn an_optional_of_a_position_a_bool_or_a_char_is_one_scalar() {
    let cpp = cpp(
        "let n: 1..=9 = read()\ndomain V = 0..n\nvar p: [V]?V = None\nvar b: [V]?Bool = None\nvar c: [V]?Char = None\n\
         p[V.first!] = V.first!.next\nb[V.first!] = true\nc[V.first!] = 'a'\n\
         print(p[V.first!] ?? V.last!, yes(b[V.first!] ?? false), c[V.first!] ?? '-')\n",
    );
    assert_eq!(decl(&cpp, "p"), "std::vector<cpl::OptPos>", "{cpp}");
    assert_eq!(decl(&cpp, "b"), "std::vector<cpl::OptBool>", "{cpp}");
    assert_eq!(decl(&cpp, "c"), "std::vector<cpl::OptChar>", "{cpp}");
    assert!(cpp.contains("using OptPos = Niche<int, int, -1>;"), "{cpp}");
    assert!(cpp.contains("using OptBool = Niche<bool, u8, 2>;"), "{cpp}");
    assert!(
        cpp.contains("using OptChar = Niche<u8, short, -1>;"),
        "{cpp}"
    );
    // No Int table was packed, so its alias is not in the file.
    assert!(!cpp.contains("OptInt"), "{cpp}");
}

#[test]
fn a_program_without_optionals_carries_none_of_the_packing() {
    let cpp = cpp("let n: 1..=9 = read()\nprint(n)\n");
    assert!(!cpp.contains("Niche"), "{cpp}");
}

#[test]
fn the_inside_of_a_small_optional_taken_as_a_place_keeps_it_unpacked() {
    let cpp = cpp("var f: ?Bool = None\nvar c: ?Char = None\n(f ??= false) = true\nprint(yes(f ?? false), c ?? '-')\n");
    assert_eq!(decl(&cpp, "f"), "std::optional<bool>", "{cpp}");
    // A Char is still packed: only the Bool is opened in place.
    assert_eq!(decl(&cpp, "c"), "cpl::OptChar", "{cpp}");
}

#[test]
fn every_optional_int_is_one_int_with_int64_min_as_none() {
    const PACKED: &str = "std::vector<cpl::OptInt>";
    // Whatever is stored: unbounded input, sums, other tables.
    assert_eq!(table("dp[V.first!] = any"), PACKED);
    assert_eq!(table("dp[V.first!] = Int.MIN"), PACKED);
    assert_eq!(
        table("dp[V.first!] = -1\ndp[V.last!] = (dp[V.first!] ?? 0) * 2"),
        PACKED
    );
    assert_eq!(table("fn f() -> Int: any\ndp[V.first!] = f()"), PACKED);
    // Used as a whole value: passed, mutated through, looped over, copied.
    assert_eq!(
        table("fn f(xs: [V]?Int) -> Int: xs[V.first!] ?? 0\nprint(f(dp))"),
        PACKED
    );
    assert_eq!(
        table("fn f(mut xs: [V]?Int):\n    xs[V.first!] = 1\nf(mut dp)"),
        PACKED
    );
    assert_eq!(table("for x in dp: print(x ?? 0)"), PACKED);
    assert_eq!(
        table("let copy = dp.clone()\nprint(copy[V.first!] ?? 0)"),
        PACKED
    );
    // A list filled by `push`, a grid, a local, a parameter and a result.
    let cpp = cpp("fn half(x: ?Int) -> ?Int: if x == Some(0): None else: x\nlet n: 0..=9 = read()\nvar xs: List<?Int> = []\nvar g: [n, n]?Int = None\nfor i in 0..n:\n    xs.push(None)\n    xs.push(i)\n    g[i, 0] = i\nlet best: ?Int = xs.pop()!\nprint(xs.len, xs[0] ?? -1, g[0, 0] ?? -1, half(best) ?? -1)\n");
    assert_eq!(decl(&cpp, "xs"), "std::vector<cpl::OptInt>", "{cpp}");
    assert_eq!(decl(&cpp, "g"), "cpl::Grid<cpl::OptInt, 2>", "{cpp}");
    assert_eq!(decl(&cpp, "best"), "cpl::OptInt", "{cpp}");
    assert!(
        cpp.contains("cpl::OptInt half(const cpl::OptInt& x)"),
        "{cpp}"
    );
    assert!(
        cpp.contains("using OptInt = Niche<ll, ll, (-0x7fffffffffffffffLL - 1)>;"),
        "{cpp}"
    );
    assert!(!cpp.contains("std::optional<ll>"), "{cpp}");
}

#[test]
fn a_bounded_optional_list_is_stored_in_four_bytes() {
    let code = cpp(
        "fn f(n: Int) -> List<?(0..2_147_483_648)>:\n    var a: List<?(0..2_147_483_648)> = List.filled(n, None)\n    a[0] = n\n    return a\nlet n: Int = read()\nlet b = f(n)\nvar c: List<?(0..100)> = [None]\nvar d: List<?Int> = [None]\nprint(b[0] ?? -1, c[0] ?? -1, d[0] ?? -1)\n",
    );
    let decl = |ty: &str| {
        code.lines()
            .filter(|l| l.trim_start().starts_with(&format!("std::vector<{ty}> ")))
            .count()
    };
    assert_eq!((decl("cpl::OptI32"), decl("cpl::OptInt")), (3, 1), "{code}");
    assert!(
        code.contains(", 0, 2147483647, \"0..2_147_483_648\")"),
        "{code}"
    );
}

#[test]
fn an_optional_struct_with_an_int_field_has_no_flag() {
    let cpp = cpp(concat!(
        "struct Edge(to: Int, w: Int)\n",
        "struct Tag(c: Char)\n",
        "var best: ?Edge = None\n",
        "best = Edge(3, 4)\n",
        "var t: ?Tag = None\n",
        "t = Tag('a')\n",
        "print(yes(best == None), yes(t == None))\n",
    ));
    assert!(cpp.contains("cpl::OptField<&Edge::to> best"), "{cpp}");
    // A `Char` has no free value, so `?Tag` keeps its flag.
    assert!(cpp.contains("std::optional<Tag> t"), "{cpp}");
}

#[test]
fn an_optional_tuple_with_a_position_inside_a_tuple_has_no_flag() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "var back: ?(from: (Int, V), via: Int) = None\n",
        "back = ((7, V.first!), 3)\n",
        "print(back!.from.0, back!.via)\n",
    ));
    assert!(
        cpp.contains("cpl::OptTuple<std::tuple<std::tuple<ll, int>, ll>, 0, 1> back"),
        "{cpp}"
    );
}

#[test]
fn an_optional_tuple_with_a_position_has_no_flag() {
    let cpp = cpp(concat!(
        "let n: Int = read()\n",
        "domain V = 0..n\n",
        "var far: ?(depth: Int, child: V) = None\n",
        "far = (3, V.first!)\n",
        "var plain: ?(Int, Int) = None\n",
        "plain = (1, 2)\n",
        "print(far!.depth, plain!.0)\n",
    ));
    assert!(
        cpp.contains("cpl::OptTuple<std::tuple<ll, int>, 1> far"),
        "{cpp}"
    );
    assert!(
        cpp.contains("template <class T, size_t... P> struct OptTuple {"),
        "{cpp}"
    );
    // No `Int` of a tuple is free to say `none`.
    assert!(
        cpp.contains("std::optional<std::tuple<ll, ll>> plain"),
        "{cpp}"
    );
}

#[test]
fn trie_row_field_niches_opt_int_and_avoids_temp_vector() {
    let out = cpp(concat!(
        "struct Trie(cnt: Int, kid: [0..2]?Int)\n",
        "var nodes = List<Trie>()\n",
        "fn add(bit: Int):\n",
        "    nodes.push(Trie(1, [0..2] _ => None))\n",
        "    let below = nodes.len - 1\n",
        "    nodes.push(Trie(1, [0..2] b => if b == bit: below else: None))\n",
        "add(read<Int>() & 1)\n",
        "print(nodes.len)\n",
    ));
    assert!(out.contains("cpl::Row<cpl::OptInt, 2> kid;"), "{out}");
    assert!(!out.contains("std::optional<ll>"), "{out}");
    assert!(
        !out.contains("-> std::vector<cpl::OptInt>"),
        "Row field comprehension must construct cpl::Row directly without a temporary std::vector: {out}"
    );
}

#[test]
fn optional_mut_param_does_not_emit_optional_of_reference() {
    let src = concat!(
        "domain V = 0..2\n",
        "fn f(seen: ?mut [V]Bool = None) -> Int:\n",
        "    0\n",
        "print(f())\n",
    );
    let out = cpp(src);
    assert!(
        !out.contains("std::optional<std::array<char, 2>&>"),
        "{out}"
    );
    assert!(out.contains("cpl::Lent<std::array<char, 2>>"), "{out}");
}

#[test]
fn an_optional_field_of_a_32_bit_range_is_an_opt_i32() {
    let field = |ty: &str| {
        let cpp = cpp(&format!(
            "type Cost = 0..=1e7\nstruct Node(best: {ty}, at: Int)\nlet nd = Node(Some(2), 1)\nprint(nd.at, nd.best ?? -1)\n"
        ));
        decl(&cpp, "best")
    };
    assert_eq!(field("?Cost"), "cpl::OptI32");
    assert_eq!(field("?(0..=1e7)"), "cpl::OptI32");
    // Past `i32::MAX` the field keeps all 64 bits.
    assert_eq!(field("?(0..=3_000_000_000)"), "cpl::OptInt");
}

#[test]
fn a_unit_optional_unwraps_as_its_flag() {
    let out = cpp(concat!(
        "fn each<R>(n: Int, f: Int -> R) -> List<R>:\n",
        "    var res: List<?R> = [None for _ in 0..n]\n",
        "    for i in 0..n: res[i] = Some(f(i))\n",
        "    [x! for x in res]\n",
        "var seen: List<Int> = []\n",
        "let n: 1..=9 = read()\n",
        "let done = each(n, i => seen.push(i))\n",
        "print(done.len, seen)\n",
    ));
    assert!(!out.contains("cpl::have("), "{out}");
}

#[test]
fn opt_max_is_one_compare_and_min_tests_a_built_some_once() {
    let body = cpp(
        "let n: Int = read()\nvar hi: List<?Int> = [None for _ in 0..n]\nvar lo: List<?Int> = [None for _ in 0..n]\nfor i in 0..n:\n    hi[i % 3] max= Some(i + n)\n    lo[i % 3] min= Some(i + n)\nprint(hi[0] ?? -1, lo[0] ?? -1)\n",
    );
    // `none` sorts first, so `max=` is the order's max, and `Some(e)` needs no test.
    assert!(body.contains("std::max<cpl::OptInt>("), "{body}");
    assert!(
        body.contains("static constexpr bool none_first = NONE < S(0);"),
        "{body}"
    );
    let solve = &body[body.find("void solve()").unwrap()..];
    assert_eq!(solve.matches(".has_value()").count(), 3, "{solve}");
}

#[test]
fn a_generic_field_keeps_its_range_argument_narrow() {
    let cpp = cpp(
        "type Small = -1000..=1000\nstruct Box<F>:\n    x: F\n    xs: List<F>\n    var os: List<?F>\n    var o: ?F\n\
         var b = Box<Small>(-1000, [1000], [None], Some(5))\nb.os.push(Some(4))\nb.o = None\n\
         print(b.x + b.xs[0], b.os.len, b.os[1]!, b.o ?? 0)\n",
    );
    assert_eq!(decl(&cpp, "x"), "int16_t", "{cpp}");
    assert_eq!(decl(&cpp, "xs"), "std::vector<int16_t>", "{cpp}");
    assert_eq!(decl(&cpp, "os"), "std::vector<cpl::OptI32>", "{cpp}");
    assert_eq!(decl(&cpp, "o"), "cpl::OptI32", "{cpp}");
}

#[test]
fn an_optional_of_an_optional_range_is_one_int32() {
    let code = cpp(
        "type Small = -1000..=1000\ntype Tag = ?Small\nstruct Box<F>:\n    x: F\n    var os: List<?F>\n    var o: ?F\n\
         var t = Box<Tag>(None, [None, Some(None)], Some(None))\nt.os.push(Some(Some(-1000)))\nt.o = Some(Some(7))\n\
         var c: List<?Tag> = [None, Some(None)]\nc.push(Some(Some(3)))\n\
         print(t.x ?? 0, t.os.len, t.o!! + c[2]!!, yes(t.os[1] == Some(None)))\n",
    );
    assert_eq!(decl(&code, "x"), "cpl::OptI32", "{code}");
    assert_eq!(decl(&code, "os"), "std::vector<cpl::OptOptI32>", "{code}");
    assert_eq!(decl(&code, "o"), "cpl::OptOptI32", "{code}");
    assert_eq!(decl(&code, "c"), "std::vector<cpl::OptOptI32>", "{code}");
    // A range reaching INT32_MIN + 1 leaves no second spare value.
    let wide = cpp(
        "type Low = -2_147_483_647..=0\ntype Tag = ?Low\nvar c: List<?Tag> = [None, Some(None)]\nc.push(Some(Some(-3)))\nprint(c.len)\n",
    );
    assert_eq!(
        decl(&wide, "c"),
        "std::vector<std::optional<cpl::OptInt>>",
        "{wide}"
    );
}

#[test]
fn a_local_moved_into_a_narrowed_field_is_packed_on_the_way_in() {
    let code = cpp(
        "type Small = -1000..=1000\nstruct Box<F>(o: F):\n    fn of(o: F) -> Box<F> = Box<F>(o)\n\
         let b = Box<?Small>.of(Some(read<Small>()))\nprint(b.o ?? 0)\n",
    );
    assert_eq!(decl(&code, "o"), "cpl::OptI32", "{code}");
    // An `OptI32` holds no heap: it is packed from `o`, not moved.
    assert!(code.contains("cpl::OptI32(o)"), "{code}");
}

#[test]
fn next_of_a_position_short_of_a_position_end_is_one_add() {
    let cpp = cpp(
        "let n: 2..=9 = read()\ndomain R = 0..n\nvar a: [R]Int = 0\nfor r in R:\n    a[r] = read()\n\
         var s = 0\nfor r in R.first!..R.last!:\n    s += a[r.next!]\n\
         for r1 in R:\n    for r2 in R:\n        if r1 < r2:\n            print(min(a[q.next!] for q in r1..r2)!)\n\
         for r in R.first!..=R.last!:\n    print(r.next ?? r)\nfor r in R:\n    if r < R.last!:\n        s += a[r.next!]\nprint(s)\n",
    );
    assert!(cpp.contains("s = cpl::iadd(s, a[r_2 + 1])"), "{cpp}");
    assert!(cpp.contains("a[q + 1]"), "{cpp}");
    // A closed range reaches the last position, and `for r in R` ends at the
    // domain's length, so their `next` keeps its check.
    assert_eq!(cpp.matches("cpl::next_pos(r_").count(), 2, "{cpp}");
}

#[test]
fn prev_of_a_position_past_a_loop_start_is_one_sub() {
    let cpp = cpp(
        "let n: 2..=9 = read()\ndomain I = 0..n\nvar a: [I]Int = 0\nfor i in I:\n    a[i] = read()\nvar s = 0\n\
         for i in I:\n    for j in i.after:\n        s += a[j.prev!]\n\
         for j in I.first!..I.last!:\n    print(j.prev ?? j)\nprint(s)\n",
    );
    assert!(cpp.contains("a[j - 1]"), "{cpp}");
    // A loop from the first position reaches a `j` with no `prev`.
    assert_eq!(cpp.matches("cpl::prev_pos(j").count(), 1, "{cpp}");
}

/// The program's own code: what follows the runtime.
fn prog(cpp: &str) -> &str {
    cpp.split_once("namespace prog {").map_or(cpp, |(_, p)| p)
}

#[test]
fn a_last_opened_where_it_is_made_is_a_check_and_a_read() {
    let cpp = cpp("let xs: List<0..9> = read()\nvar s = 0\nif let a = xs.last: s += a\nlet b = xs.last!\nlet c = xs.first ?? 7\nprint(s, b, c)\n");
    let p = prog(&cpp);
    assert!(
        p.contains("if (!xs.empty()) {") && p.contains("= xs.back();"),
        "{p}"
    );
    assert!(p.contains("(!xs.empty() ? (void)0 : cpl::fail(\"`!` on `None`: there is no value here\"), xs.back())"), "{p}");
    assert!(p.contains("!xs.empty() ? xs.front() : 7"), "{p}");
    for built in ["cpl::last(", "cpl::first(", "has_value()", "cpl::have("] {
        assert!(!p.contains(built), "{built} builds an optional:\n{p}");
    }
}

#[test]
fn a_domain_position_mapped_and_tested_builds_no_position_optional() {
    let cpp = cpp("let w: 1..=9 = read()\ndomain Col = 0..w\nfn step(c: Col, d: Int) -> ?(Col, Int) = Col.at(c + d).map(n => (n, d))\nlet c: Col = Col.first!\nif let (n, d) = step(c, 1): print(n.idx, d)\n");
    let p = prog(&cpp);
    assert!(p.contains(" < Col_len) {"), "{p}");
    assert!(
        !p.contains("cpl::pos_try_ord(") && !p.contains("cpl::OptPos t"),
        "{p}"
    );
}

#[test]
fn an_if_chain_of_cells_tests_its_cell_once_after_the_chain() {
    let src = "let h: 1..=9 = read()\nlet w: 1..=9 = read()\ndomain Row = 0..h\ndomain Col = 0..w\ntype Cell = (Row, Col)\n\
         fn step(p: Cell, m: Int) -> ?Cell =\n    if 10 / m == 1: Col.at(p.1 - 1).map(c => (p.0, c))\n    elif m == 2: Col.at(p.1 + 1).map(c => (p.0, c))\n    else: Row.at(p.0 + 1).map(r => (r, p.1))\n\
         fn mixed(p: Cell, m: Int) -> ?Cell =\n    if m == 0: Col.at(p.1 - 1).map(c => (p.0, c))\n    else: None\n\
         var at: Cell = (Row(0), Col(w - 1))\nfor m in 1..4:\n    if let next = step(at, m): at = next\n\
         print(at.0.idx, at.1.idx, yes(mixed(at, 1) == None))\n";
    let cpp = cpp(src);
    let p = prog(&cpp);
    let body = |f: &str| {
        (p.split(&format!("bool {f}(")).nth(1))
            .and_then(|s| s.split("\n}\n").next())
            .unwrap_or("")
            .to_string()
    };
    let step = body("step");
    assert_eq!(step.matches("cpl_out = ").count(), 1, "{step}");
    assert_eq!(step.matches("_len").count(), 2, "{step}");
    assert!(
        step.contains("< (cpl::u64)Row_len) & ((cpl::u64)")
            && step.contains("< (cpl::u64)Col_len)"),
        "{step}"
    );
    // The arms' conditions still run first and in order, so a trap in one
    // (here `10 / m`) comes before any cell is tested.
    let (div, test) = (step.find("cpl::divn(10, m)"), step.find("_len"));
    assert!(div.is_some() && div < test, "{step}");
    // An arm that is not a cell keeps every arm's own test.
    let mixed = body("mixed");
    assert!(!mixed.contains("(cpl::u64)"), "{mixed}");
}

#[test]
fn a_cell_fn_tested_by_its_caller_builds_no_opt_tuple() {
    let src = "let h: 1..=9 = read()\nlet w: 1..=9 = read()\ndomain Row = 0..h\ndomain Col = 0..w\ntype Cell = (Row, Col)\n\
         fn step(p: Cell, m: Int) -> ?Cell =\n    if m == 0: Col.at(p.1 - 1).map(c => (p.0, c))\n    else: Row.at(p.0 + 1).map(r => (r, p.1))\n\
         fn kept(p: Cell) -> ?Cell = Row.at(p.0 + 1).map(r => (r, p.1))\n\
         var at: Cell = (Row(0), Col(w - 1))\nfor m in 0..4:\n    if let next = step(at, m % 2): at = next\n\
         print(at.0.idx, at.1.idx, yes(kept(at) == None))\n";
    let cpp = cpp(src);
    let p = prog(&cpp);
    assert!(
        p.contains("bool step(const std::tuple<int, int>& p, ll m, std::tuple<int, int>& cpl_out)"),
        "{p}"
    );
    assert!(p.contains("if (::prog::step(at, "), "{p}");
    let body = p
        .split("bool step(")
        .nth(1)
        .and_then(|s| s.split("\n}\n").next())
        .unwrap_or("");
    assert!(
        !body.contains("OptTuple") && !body.contains("OptPos"),
        "{body}"
    );
    // A call the caller compares, not tests, keeps the optional.
    assert!(
        p.contains("cpl::OptTuple<std::tuple<int, int>, 0> kept("),
        "{p}"
    );
}

#[test]
fn a_position_stepped_by_k_checks_only_the_side_it_can_leave() {
    let cpp = cpp("let w: 1..=9 = read()\ndomain Col = 0..w\nlet c: Col = Col.first!\nlet k: 0..=9 = read()\nif let l = Col.at(c - 1): print(l.idx)\nif let r = Col.at(c + k): print(r.idx)\n");
    let p = prog(&cpp);
    let checks: Vec<&str> = p
        .lines()
        .filter(|l| l.trim_start().starts_with("if ("))
        .collect();
    assert!(
        checks
            .iter()
            .any(|l| l.contains(">= 0") && !l.contains("Col_len")),
        "{p}"
    );
    assert!(
        checks
            .iter()
            .any(|l| l.contains("< Col_len") && !l.contains(">= 0")),
        "{p}"
    );
}

#[test]
fn a_profiled_build_counts_the_line_that_tests_an_into_fn() {
    let src =
        "let n: Int = read()\nfn half(x: Int) -> ?Int = if x % 2 == 0: Some(x / 2) else: None\n\
         var s = 0\nfor i in 0..n:\n    if let h = half(i): s += h\nprint(s)\n";
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", src);
    let analysis = db.analyze("t.cp").expect("the file was just set");
    let program = cpl_ir::lower(&analysis).expect("lowers");
    let (profiled, spans) =
        crate::emit_profiled(&program, "t.cp", &crate::Target::default(), "t.prof");
    assert!(profiled.contains("ll& cpl_out)"), "{profiled}");
    let at = src.find("half(i)").unwrap() as u32;
    assert!(
        spans.iter().any(|s| s.start <= at && at < s.end),
        "no site counts `if let h = half(i)`: {spans:?}\n{profiled}"
    );
}

#[test]
fn the_max_of_optional_positions_is_one_std_max_and_their_min_one_unsigned_compare() {
    let cpp = cpp(
        "let n: 1..=9 = read()\ndomain I = 0..n\nlet a: [I]?I = [if i.ord % 3 == 0: None else: Some(i) for i in I]\n\
         print(a.fold(max), a.fold(min))\nvar b: ?I = None\nfor x in a: b = max(b, x)\nprint(b)\n",
    );
    assert!(cpp.contains("std::max<cpl::OptPos>(a_2, b_2)"), "{cpp}");
    assert!(cpp.contains("b = std::max<cpl::OptPos>(b, x)"), "{cpp}");
    assert!(cpp.contains("cpl::opt_min("), "{cpp}");
    assert!(!cpp.contains("has_value() ? ("), "{cpp}");
}

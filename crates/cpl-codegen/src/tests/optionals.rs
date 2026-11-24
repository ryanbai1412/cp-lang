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
    let cpp = cpp("fn half(x: ?Int) -> ?Int: x\nlet n: 0..=9 = read()\nvar xs: List<?Int> = []\nvar g: [n, n]?Int = None\nfor i in 0..n:\n    xs.push(None)\n    xs.push(i)\n    g[i, 0] = i\nlet best: ?Int = xs.pop()!\nprint(xs.len, xs[0] ?? -1, g[0, 0] ?? -1, half(best) ?? -1)\n");
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
    assert!(!out.contains("std::optional<std::vector<char>&>"), "{out}");
    assert!(out.contains("std::optional<std::vector<char>>"), "{out}");
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

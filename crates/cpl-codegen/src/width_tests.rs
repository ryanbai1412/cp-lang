//! A position's C++ integer is as wide as its domain's proven size.

use crate::tests::cpp;

#[test]
fn a_position_of_a_sum_domain_past_32_bits_is_an_ll() {
    let enum_of = |hi: &str| {
        cpp(&format!(
            "let n: 1..={hi} = read()\ndomain C = 1..=n\nenum Link:\n    A(a: C, b: C)\n    B(a: C, b: C)\nmatch B(C.last!, C.last!):\n    A(a, b) => print(a.ord + b.ord)\n    B(a, b) => print(-a.ord - b.ord)\n"
        ))
    };
    // 2 * 1e5 * 1e5 positions need 64 bits; 2 * 1e3 * 1e3 do not.
    assert!(
        enum_of("1e5").contains("ll t1 = (ll)"),
        "{}",
        enum_of("1e5")
    );
    assert!(
        enum_of("1000").contains("int t1 = (int)"),
        "{}",
        enum_of("1000")
    );
}

#[test]
fn a_heap_of_position_pairs_past_32_bits_keys_by_ll() {
    let heap_of = |hi: &str| {
        cpp(&format!(
            "domain D = read(1..={hi})\nvar pq = MinHeap<(D, D)>()\npq.push((D.last!, D.last!))\nlet (x, y) = pq.pop()!\nprint(yes(x == D.last!), yes(y == D.last!))\n"
        ))
    };
    assert!(
        heap_of("500_000").contains("std::vector<ll>"),
        "{}",
        heap_of("500_000")
    );
    assert!(
        heap_of("40_000").contains("std::vector<int>"),
        "{}",
        heap_of("40_000")
    );
}

#[test]
fn a_checked_build_traps_on_a_plain_domain_past_32_bits() {
    let cpp = cpp("let n: Int = read()\ndomain C = 0..n\nprint(C.count)\n");
    assert!(cpp.contains("C_len > 2147483647"), "{cpp}");
}

#[test]
fn a_list_of_a_small_domain_holds_each_position_in_a_byte() {
    let small = cpp(concat!(
        "domain L = 'a'..='z'\n",
        "let xs = [L(c) for c in read<Str>()]\n",
        "print(xs[0].ord + xs.len)\n",
    ));
    assert!(small.contains("std::vector<cpl::u8>"), "{small}");
    // Braces do not narrow an `int` position to a byte without a cast.
    let literal = cpp(concat!(
        "domain L = 'a'..='z'\n",
        "let (a, b) = read<(Char, Char)>()\n",
        "let xs = [L(a), L(b)]\n",
        "print(xs[0].ord + xs[1].ord)\n",
    ));
    assert!(
        literal.contains("std::vector<cpl::u8>{(cpl::u8)("),
        "{literal}"
    );
    // 256 positions and the end bound are 257 values: past a byte.
    let wide = cpp(concat!(
        "domain L = [256]\n",
        "let n: 1..=1000 = read()\n",
        "let xs = [L.at(i % 256)! for i in 0..n]\n",
        "print(xs[0].ord + xs.len)\n",
    ));
    assert!(!wide.contains("cpl::u8>"), "{wide}");
    assert!(wide.contains("std::vector<int>"), "{wide}");
}

/// `R`, `C` and `P = (R, C)` with at most `hi` rows and columns, and `body`.
fn product_of(hi: &str, body: &str) -> String {
    cpp(&format!(
        "domain R = read(1..={hi})\ndomain C = read(1..={hi})\ndomain P = (R, C)\nlet i: Int = read()\n{body}"
    ))
}

#[test]
fn an_ordinal_cast_into_a_wide_product_checks_64_bits() {
    let body = "var a: [P]Int = 0\na[P.nth(i)] = 1\nprint(a[P.nth(i)])\n";
    let wide = product_of("100_000", body);
    assert!(wide.contains("cpl::pos_wide(i, P_len)"), "{wide}");
    let narrow = product_of("1000", body);
    assert!(narrow.contains("cpl::pos(i, P_len)"), "{narrow}");
}

#[test]
fn a_product_of_exactly_i32_max_positions_stays_an_int() {
    let at = |hi: &str| {
        cpp(&format!(
            "domain R = read(1..={hi})\ndomain One = 0..1\ndomain P = (R, One)\nvar pq = MinHeap<P>()\npq.push(P.first!)\nprint(yes(pq.pop()! == P.first!))\n"
        ))
    };
    let max = at("2147483647");
    assert!(max.contains("std::vector<int>"), "{max}");
    let past = at("2147483648");
    assert!(past.contains("std::vector<ll>"), "{past}");
}

#[test]
fn a_bool_or_enum_part_counts_its_cases() {
    let with = |part: &str| {
        cpp(&format!(
            "enum E:\n    X\n    Y\n    Z\ndomain D = read(1..=1000)\ndomain P = (D, {part})\nvar pq = MinHeap<P>()\npq.push(P.first!)\nprint(yes(pq.pop()! == P.first!))\n"
        ))
    };
    for part in ["Bool", "E"] {
        let out = with(part);
        assert!(out.contains("std::vector<int>"), "{part}: {out}");
    }
}

#[test]
fn explain_names_each_domains_width_and_the_bound_behind_it() {
    let source = concat!(
        "domain R = read(1..=100_000)\ndomain C = read(1..=100_000)\ndomain P = (R, C)\n",
        "domain Q = read(1..=1000)\ndomain W = (Q, Q)\ndomain S = 0..5\n",
        "domain D = 0..Q.count * 3\nlet i: Int = read()\nvar d: [D]Int = 0\nd[D.nth(i)] = 4\n",
        "var a: [P]Int = 0\na[P.nth(i)] = 1\nvar w: [W]Int = 0\nw[W.nth(i)] = 2\n",
        "var b: [S]Int = 0\nb[S.nth(i % 5)] = 3\n",
        "print(a[P.nth(i)] + b[S.nth(i % 5)] + w[W.nth(i)] + d[D.nth(i)])\n",
    );
    let mut db = cpl_check::Session::new();
    db.set_file("t.cp", source);
    let program = cpl_ir::lower(&db.analyze("t.cp").expect("set")).expect("lowers");
    let plain = "`int` positions: a plain domain holds at most i32::MAX";
    let bounded = "`int` positions: a bound keeps its count under i32::MAX";
    let wide =
        "`ll` positions: its count multiplies other counts and no bound keeps it under i32::MAX";
    assert_eq!(
        crate::explain_lowered(&program).specializations,
        [
            format!("domain R -> {plain}"),
            format!("domain C -> {plain}"),
            format!("domain P -> {wide}"),
            format!("domain Q -> {plain}"),
            format!("domain W -> {bounded}"),
            format!("domain S -> {plain}; 5 on every run, so an array over it is a `std::array`"),
            format!("domain D -> {bounded}"),
        ]
    );
}

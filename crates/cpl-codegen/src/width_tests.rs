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

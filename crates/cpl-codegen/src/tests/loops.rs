//! How range loops are written in C++.

use super::cpp;

#[test]
fn an_inclusive_loop_to_a_bound_below_int_max_is_a_plain_loop() {
    let open = cpp("let n: Int = read()\nfor i in 0..=n:\n    print(i)\n");
    assert!(open.contains("i_go"), "{open}");
    let len = cpp("let xs = [1, 2, 3]\nfor i in 0..=xs.len:\n    print(i)\n");
    assert!(len.contains("i <= "), "{len}");
    assert!(!len.contains("i_go"), "{len}");
    let tuple = cpp(
        "domain D = 0..5\nlet n: Int = read()\nlet (k, m) = (n, D.count)\nfor i in 1..=m - 1:\n    print(i, k)\n",
    );
    assert!(!tuple.contains("i_go"), "{tuple}");
    let quotient = cpp(
        "let (n, m): (1..=10, 2..=3000) = read()\nfor j in 1..n:\n    for d in 1..=m / (j + 1):\n        print(d)\n",
    );
    assert!(!quotient.contains("d_go"), "{quotient}");
    let wrapping = cpp(
        "let m: 2..=3000 = read()\nunchecked:\n    for j in 1..m:\n        for d in 1..=m / (j + 1):\n            print(d * (j + 1))\n",
    );
    assert!(
        !wrapping.contains("d_go") && !wrapping.contains("ediv"),
        "{wrapping}"
    );
    assert!(
        !wrapping.contains("uint64_t(j)"),
        "a sum that can't wrap is plain: {wrapping}"
    );
}

#[test]
fn a_map_scanned_only_into_a_list_sorted_by_value_is_a_hash_table() {
    let source = concat!(
        "let n: 1..=1000 = read()\n",
        "let a: [0..n]Int = read()\n",
        "var freq: Map<Int, Int> = Map()\n",
        "for x in a: freq[x] += 1\n",
        "let odd = [cnt for (x, cnt) in freq.items if x % 2 == 1].sorted(by: c => -c)\n",
        "print(odd)\n",
    );
    let hashed = cpp(source);
    assert!(!hashed.contains("std::map<ll, ll> freq"), "{hashed}");
    let ordered = cpp(&source.replace("sorted(by: c => -c)", "rev"));
    assert!(ordered.contains("std::map<ll, ll> freq"), "{ordered}");
}

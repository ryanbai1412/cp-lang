//! How narrow the C++ stores a declared range.

use super::cpp;

#[test]
fn a_struct_field_of_a_range_alias_narrows_like_the_range() {
    let out = cpp("type Sign = -1..=1
struct B:
    var s: Sign
    var t: -1..=1
var b = B(1, -1)
b.s = -b.s
print(b.s, b.t)
");
    assert!(
        out.contains("int8_t s = 0;"),
        "a `Sign` field must take a byte:\n{out}"
    );
    assert!(out.contains("int8_t t = 0;"), "{out}");
}

#[test]
fn a_struct_array_over_a_range_alias_stays_inline() {
    let out = cpp("type Sign = -1..=1
struct T:
    var n: [Sign]Int
var t = T([4, 5, 6])
t.n[-1] += 1
print(t.n[-1])
");
    assert!(
        out.contains("cpl::Row<ll, 3> n;"),
        "`[Sign]Int` has 3 cells:\n{out}"
    );
}

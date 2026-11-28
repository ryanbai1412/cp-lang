//! The `Eq`, `Ord`, `Show` and `Hash` a struct gets field by field, as library code on
//! `fields` (decision 28.7), for an `impl` to call: `impl Ord for Job: fn lt(a, b) =
//! lt_fields(b, a)` orders the other way, and `impl Hash for P: fn hash(self) =
//! hash_fields(self)` is a struct's whole `Hash`. Each costs \(O(\mathit{fields})\) plus what its fields'
//! own operations cost.
//!
//! start: `eq_fields`, `lt_fields`, `show_fields`, `read_fields`, `hash_fields`
//! example: `struct P(a: Int, b: Int); eq_fields(P(1, 2), P(1, 2)) == true`
use std.hash.{splitmix64}
use std.rand.{Random}

/// `a == b` field by field, in declaration order, stopping at the first field that differs.
///
/// cost: \(O(\mathit{fields})\)
/// empty: `struct E(); eq_fields(E(), E()) == true`
/// traps: never
/// example: `struct P(a: Int, b: Int); eq_fields(P(1, 2), P(1, 2)) == true`
/// example: `struct P(a: Int, b: Int); eq_fields(P(1, 2), P(1, 3)) == false`
pub fn eq_fields<T>(a: T, b: T) -> Bool:
    for (x, y) in fields(a, b):
        if x != y: return false
    true

/// `a < b` by the first field that differs, in declaration order (lexicographic).
///
/// cost: \(O(\mathit{fields})\)
/// empty: `struct E(); lt_fields(E(), E()) == false`
/// ties: `struct P(a: Int, b: Int); lt_fields(P(1, 2), P(1, 2)) == false`
/// traps: never
/// example: `struct P(a: Int, b: Int); lt_fields(P(1, 2), P(1, 3)) == true`
/// example: `struct P(a: Int, b: Int); lt_fields(P(2, 0), P(1, 3)) == false`
pub fn lt_fields<T>(a: T, b: T) -> Bool:
    for (x, y) in fields(a, b):
        if x != y: return x < y
    false

/// The fields of `a` in declaration order, one space apart, each shown as `print` would.
///
/// cost: \(O(\mathit{fields})\) plus the length of the result
/// empty: `struct E(); show_fields(E()) == ""`
/// order: declaration order
/// traps: never
/// example: `struct P(a: Int, b: Int); show_fields(P(3, 4)) == "3 4"`
pub fn show_fields<T>(a: T) -> Str:
    var s = ""
    var first = true
    for f in fields(a):
        s = if first: "{f}" else: "{s} {f}"
        first = false
    s

/// A `T` read from input field by field, each with its own type's `read()`, in
/// declaration order: `read_fields<Edge>()` with no `impl` of its own.
///
/// cost: \(O(\mathit{fields})\) plus the tokens read
/// empty: `struct E(); let e = read_fields<E>(); read<Int>() == 4` (stdin `4`)
/// order: declaration order
/// traps: when input runs out or a token does not parse as its field's type
/// example: `struct P(a: Int, b: Int); read_fields<P>().b == 5` (stdin `4 5`)
pub fn read_fields<T>() -> T = T(read() for _ in fields(T))

/// A random `T`, each field drawn by its own type's `gen()`:
/// `impl Random for P: fn gen() -> P = gen_fields<P>()`.
///
/// cost: \(O(\mathit{fields})\) plus what each field's draw costs
/// empty: `struct E(); let e = gen_fields<E>(); gen(1..=1) == 1`
/// order: declaration order
/// traps: never
/// example: `struct P(a: 1..=9); gen_fields<P>().a <= 9`
pub fn gen_fields<T>() -> T = T(gen() for _ in fields(T))

/// The fields' hashes combined in declaration order, so equal structs hash equal and
/// reordered field values usually do not.
///
/// cost: \(O(\mathit{fields})\) plus what each field's `hash` costs
/// empty: `struct E(); struct F(); hash_fields(E()) == hash_fields(F())`
/// order: declaration order
/// traps: never
/// example: `struct P(a: Int, b: Int); hash_fields(P(1, 2)) == hash_fields(P(1, 2))`
/// example: `struct P(a: Int, b: Int); hash_fields(P(1, 2)) != hash_fields(P(2, 1))`
pub fn hash_fields<T>(a: T) -> U64:
    var h = U64(0)
    for f in fields(a): h = splitmix64(h ^ f.hash())
    h

@test
fn field_derives_agree_with_the_compilers_structural_ones():
    struct P(a: Int, b: Char, c: Bool)
    let xs = [P(1, 'a', false), P(1, 'a', true), P(1, 'b', false), P(0, 'z', true)]
    for x in xs:
        for y in xs:
            assert eq_fields(x, y) == (x == y)
            assert lt_fields(x, y) == (x < y)
    assert show_fields(xs[3]) == "0 z YES"
    assert hash_fields(xs[0]) == hash_fields(P(1, 'a', false))
    assert hash_fields(xs[0]) != hash_fields(xs[1])

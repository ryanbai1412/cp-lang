//! The `Eq`, `Ord`, `Show` and `Hash` a struct gets field by field, as library code on
//! `fields` (decision 28.7), for an `impl` to call: `impl Ord for Job: fn lt(a, b) =
//! lt_fields(b, a)` orders the other way, and `impl Hash for P: fn hash(self) =
//! hash_fields(self)` is a struct's whole `Hash`. Each costs \(O(\mathit{fields})\) plus what its fields'
//! own operations cost.

/// `a == b` field by field, in declaration order, stopping at the first field that differs.
///
/// cost: \(O(\mathit{fields})\)
/// empty: `true` for a struct with no fields
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
/// empty: `false` for a struct with no fields
/// ties: `false` when every field is equal
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
/// empty: `""` for a struct with no fields
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
/// empty: reads nothing for a struct with no fields
/// order: declaration order
/// traps: when input runs out or a token does not parse as its field's type
/// example: `struct P(a: Int, b: Int); read_fields<P>().b == 5` (stdin `4 5`)
pub fn read_fields<T>() -> T = T(read() for _ in fields(T))

/// A value's 64-bit hash: equal values hash equal. `Int`, `U64`, `Char`, `Bool` and `Str`
/// have it here (splitmix64, decision 28.8); a struct gets it from `hash_fields`.
pub trait Hash:
    fn hash(self) -> U64

/// splitmix64's finalizer: every input bit moves about half the output bits.
fn mix64(z0: U64) -> U64:
    let top = U64(1) << 63
    var z = z0 + (U64(0x1e3779b97f4a7c15) | top)
    z = (z ^ (z >> 30)) * (U64(0x3f58476d1ce4e5b9) | top)
    z = (z ^ (z >> 27)) * (U64(0x14d049bb133111eb) | top)
    z ^ (z >> 31)

impl Hash for Int:
    /// cost: \(O(1)\)
    /// example: `5.hash() == 5.hash()`
    fn hash(self) -> U64 = mix64(U64(self))

impl Hash for U64:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = mix64(self)

impl Hash for Char:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = mix64(U64(Int(self)))

impl Hash for Bool:
    /// cost: \(O(1)\)
    fn hash(self) -> U64 = mix64(if self: U64(1) else: U64(0))

impl Hash for Str:
    /// cost: \(O(\mathit{length})\)
    /// example: `"ab".hash() != "ba".hash()`
    fn hash(self) -> U64:
        var h = U64(self.len)
        for c in self: h = mix64(h ^ U64(Int(c)))
        h

/// The fields' hashes combined in declaration order, so equal structs hash equal and
/// reordered field values usually do not.
///
/// cost: \(O(\mathit{fields})\) plus what each field's `hash` costs
/// empty: the same fixed value for every struct with no fields
/// order: declaration order
/// traps: never
/// example: `struct P(a: Int, b: Int); hash_fields(P(1, 2)) == hash_fields(P(1, 2))`
/// example: `struct P(a: Int, b: Int); hash_fields(P(1, 2)) != hash_fields(P(2, 1))`
pub fn hash_fields<T>(a: T) -> U64:
    var h = U64(0)
    for f in fields(a): h = mix64(h ^ f.hash())
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

//! Polynomial hashes of sequences, as values that can be joined, cut and
//! compared. Proposed for `std.hashing` (friction-backlog-batch-10, T1).
//!
//! A `Hash` stands for a whole sequence: equal sequences have equal hashes,
//! and two different sequences collide with probability about `len / 1e18`
//! (two residues modulo two primes near 1e9, with bases picked at random at
//! startup so that no input can be prepared against them).

type A = Mod<1000000007>
type B = Mod<998244353>

let base_a = A(rand(1000, 1000000000))
let base_b = B(rand(1000, 990000000))
var pow_a = [A.one]
var pow_b = [B.one]

fn grow(n: Int):
    while pow_a.len <= n:
        pow_a.push(pow_a.last * base_a)
        pow_b.push(pow_b.last * base_b)

/// The hash of a sequence, with its length. `x + y` is the hash of `x` followed by `y`. O(1)
/// amortized: the powers of the base are kept in a table that grows with the longest hash seen.
pub struct Hash(len: Int, a: A, b: B):
    /// The empty sequence.
    let empty = Hash(0, A.zero, B.zero)

    /// A sequence of one item. Items are `Int`s; a `Char` is its code.
    pub fn one(x: Int) -> Hash = Hash(1, A.new(x + 1), B.new(x + 1))

    /// The hash of a whole sequence. O(n).
    pub fn all(xs: List<Int>) -> Hash:
        var h = Hash.empty
        for x in xs: h += Hash.one(x)
        h

    /// `self` without its first `head.len` items, which must hash to `head`.
    pub fn after(self, head: Hash) -> Hash:
        let rest = len - head.len
        grow(rest)
        Hash(rest, a - head.a * pow_a[rest], b - head.b * pow_b[rest])

impl Add for Hash:
    fn add(x, y):
        grow(y.len)
        Hash(x.len + y.len, x.a * pow_a[y.len] + y.a, x.b * pow_b[y.len] + y.b)

/// The hashes of every block of a fixed sequence. Built in O(n); `of` is O(1).
pub struct StrHash:
    pre: List<Hash>

    pub fn of_items(xs: List<Int>) -> StrHash:
        var pre = [Hash.empty]
        for x in xs: pre.push(pre.last + Hash.one(x))
        StrHash(pre)

    pub fn new(s: Str) -> StrHash = StrHash.of_items([Int(c) for c in s])

    /// The number of items in the sequence.
    pub @get fn len(self) -> Int = pre.len - 1

    /// The hash of the block `r`, a range of positions.
    pub fn of(self, r: Range<Int>) -> Hash = pre[r.end].after(pre[r.start])

    /// How many items the blocks starting at `i` and `j` share before they differ. O(log n).
    pub fn lcp(self, i: Int, j: Int) -> Int:
        var (lo, hi) = (0, self.len - max(i, j))
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.of(i..i + mid) == self.of(j..j + mid): lo = mid
            else: hi = mid - 1
        lo

    /// How many items the blocks ending just before `i` and `j` share, counted backwards. O(log n).
    pub fn lcs(self, i: Int, j: Int) -> Int:
        var (lo, hi) = (0, min(i, j))
        while lo < hi:
            let mid = (lo + hi + 1) / 2
            if self.of(i - mid..i) == self.of(j - mid..j): lo = mid
            else: hi = mid - 1
        lo

@test
fn joining_and_cutting_agree_with_blocks():
    let h = StrHash.new("abracadabra")
    assert h.of(0..4) == h.of(7..11)
    assert h.of(0..4) != h.of(1..5)
    assert h.of(0..4) + h.of(4..7) == h.of(0..7)
    assert h.of(0..7).after(h.of(0..4)) == h.of(4..7)
    assert Hash.all([Int(c) for c in "abra"]) == h.of(7..11)
    assert h.of(3..3) == Hash.empty

@test
fn lcp_and_lcs_count_shared_items():
    let h = StrHash.new("abracadabra")
    assert h.lcp(0, 7) == 4
    assert h.lcp(0, 3) == 1
    assert h.lcp(1, 2) == 0
    assert h.lcs(4, 11) == 4
    assert h.lcs(1, 4) == 1
    assert h.lcs(0, 5) == 0

//! String algorithms. They only compare elements, so they work on a `Str`
//! (ASCII bytes) and on any `List<T>` alike. Positions are 0-based.

/// `z[i]` is the length of the longest common prefix of `s` and `s[i..]`; `z[0]` is `s.len`. O(n).
pub fn z_function<T>(s: List<T>) -> List<Int>:
    let n = s.len
    var z = List.filled(n, 0)
    if n == 0: return z
    z[0] = n
    var (l, r) = (0, 0)
    for i in 1..n:
        var k = if i < r: min(r - i, z[i - l]) else: 0
        while i + k < n and s[k] == s[i + k]: k += 1
        z[i] = k
        if i + k > r: (l, r) = (i, i + k)
    z

/// `p[i]` is the length of the longest proper prefix of `s[..=i]` that is also its suffix. O(n).
pub fn prefix_function<T>(s: List<T>) -> List<Int>:
    var p = List.filled(s.len, 0)
    for i in 1..s.len:
        var k = p[i - 1]
        while k > 0 and s[i] != s[k]: k = p[k - 1]
        if s[i] == s[k]: k += 1
        p[i] = k
    p

/// The start of every occurrence of `pat` in `text`, in increasing order (Knuth-Morris-Pratt).
/// O(text.len + pat.len). An empty pattern occurs at every position, including the end.
pub fn find_all<T>(text: List<T>, pat: List<T>) -> yields Int:
    if pat.is_empty:
        for i in 0..=text.len: yield i
        return
    let p = prefix_function(pat)
    var k = 0
    for i in 0..text.len:
        while k > 0 and text[i] != pat[k]: k = p[k - 1]
        if text[i] == pat[k]: k += 1
        if k == pat.len:
            yield i + 1 - k
            k = p[k - 1]

/// The length of the shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` for all `i`. O(n).
pub fn period<T>(s: List<T>) -> Int = if s.is_empty: 0 else: s.len - prefix_function(s).last

/// The starts of the suffixes of `s` in sorted order, by prefix doubling. O(n log^2 n).
pub fn suffix_array<T>(s: List<T>) -> List<Int>:
    let n = s.len
    var sa = [i for i in 0..n]
    if n <= 1: return sa
    // ranks of single elements, by sorting positions on the element
    sort(mut sa, by: i => s[i])
    var rank = List.filled(n, 0)
    for j in 1..n: rank[sa[j]] = rank[sa[j - 1]] + (if s[sa[j - 1]] < s[sa[j]]: 1 else: 0)
    var k = 1
    while k < n and rank[sa[n - 1]] < n - 1:
        // order by (rank[i], rank[i + k]); a suffix shorter than k sorts first
        let key = [rank[i] * (n + 1) + (if i + k < n: rank[i + k] + 1 else: 0) for i in 0..n]
        sort(mut sa, by: i => key[i])
        var next = List.filled(n, 0)
        for j in 1..n: next[sa[j]] = next[sa[j - 1]] + (if key[sa[j - 1]] < key[sa[j]]: 1 else: 0)
        rank = next
        k *= 2
    sa

/// `lcp[j]` is the length of the longest common prefix of the suffixes at `sa[j]` and `sa[j + 1]`
/// (Kasai). O(n); the result has `n - 1` entries.
pub fn lcp_array<T>(s: List<T>, sa: List<Int>) -> List<Int>:
    let n = s.len
    var lcp = List.filled(max(n - 1, 0), 0)
    var rank = List.filled(n, 0)
    for j in 0..n: rank[sa[j]] = j
    var k = 0
    for i in 0..n:
        if rank[i] == n - 1:
            k = 0
            continue
        let j = sa[rank[i] + 1]
        while i + k < n and j + k < n and s[i + k] == s[j + k]: k += 1
        lcp[rank[i]] = k
        if k > 0: k -= 1
    lcp

// ---- generalized rolling hash

const RH_M1 = 1_000_000_007
const RH_M2 = 1_000_000_009

/// A 64-bit double-mod polynomial hash value (`Copy + Eq + Ord + Hash`).
/// Can itself be fed into another `RollingHash` (e.g. hashing rows into a column hash for 2D grids).
pub struct HashVal(h1: Int, h2: Int):
    /// Mix both 31-bit halves into one integer token so a `RollingHash` can hash `[D]HashVal`.
    pub @get fn hash_token(self) -> Int = (h1 << 31) ^ h2

/// Elements accepted by `RollingHash`: `Char`, `Int`, `Bool`, `HashVal`, or any user type
/// implementing `hash_token`.
pub trait Hashable:
    @get fn hash_token(self) -> Int = Int(self)

@get fn ord(self: Int) -> Int = self

/// Generalized double-mod polynomial rolling hash over `Str`, `List<T>`, or any `[D]T`
/// (`Char`, `Int`, `Bool`, `HashVal`).
///
/// Built in O(n); substring queries `.get(l..r)` / `.slice(l, r)` and `.combine(a, b, b_len)`
/// are O(1). Each constructor draws independent random salted bases unless `bases:` is passed
/// explicitly, so 2D row and column hashes never collide on transposes.
pub struct RollingHash:
    bases: (Int, Int)
    p1: List<Int>
    p2: List<Int>
    pow1: List<Int>
    pow2: List<Int>

    pub fn new<S>(seq: S, bases: ?(Int, Int) = none) -> RollingHash:
        let (b1, b2) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        var p1 = [0]
        var p2 = [0]
        var pow1 = [1]
        var pow2 = [1]
        for x in seq:
            let t = x.hash_token
            let x1 = t % (RH_M1 - 1) + 1
            let x2 = t % (RH_M2 - 1) + 1
            p1.push((p1[p1.len - 1] * b1 + x1) % RH_M1)
            p2.push((p2[p2.len - 1] * b2 + x2) % RH_M2)
            pow1.push(pow1[pow1.len - 1] * b1 % RH_M1)
            pow2.push(pow2[pow2.len - 1] * b2 % RH_M2)
        RollingHash((b1, b2), p1, p2, pow1, pow2)

    /// Build a `RollingHash` reusing another instance's salted `bases` so their hashes compare directly.
    pub fn with_bases<S>(seq: S, bases: (Int, Int)) -> RollingHash = RollingHash.new(seq, bases: bases)

    /// Number of hashed elements.
    pub @get fn len(self) -> Int = p1.len - 1

    /// Hash of the entire sequence. O(1).
    pub @get fn all(self) -> HashVal = self.slice(0, p1.len - 1)

    /// Hash of the half-open 0-based slice `[l, r)`, or `HashVal(0, 0)` when `l >= r`. O(1).
    pub fn slice(self, l: Int, r: Int) -> HashVal:
        if l >= r: return HashVal(0, 0)
        assert 0 <= l and r <= p1.len - 1
        let d = r - l
        HashVal((p1[r] - p1[l] * pow1[d]) % RH_M1, (p2[r] - p2[l] * pow2[d]) % RH_M2)

    /// Hash of a range (`Range<Int>` such as `l..r`, `l..=r`, `..k`, or `Range<D>` on `[D]T`). O(1).
    pub fn get<R>(self, r: R) -> HashVal:
        if r.is_empty: return HashVal(0, 0)
        self.slice(r.lo.ord, r.hi.ord)

    /// Concatenate hashes `a` and `b` where `b` has length `b_len`, in O(1).
    pub fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal:
        HashVal((a.h1 * pow1[b_len] + b.h1) % RH_M1, (a.h2 * pow2[b_len] + b.h2) % RH_M2)

// ---- tests

fn random_word(n: Int, letters: Int) -> List<Char> = [Char(Int('a') + rand(0, letters - 1)) for _ in 0..n]

fn common_prefix<T>(s: List<T>, a: Int, b: Int) -> Int:
    var k = 0
    while a + k < s.len and b + k < s.len and s[a + k] == s[b + k]: k += 1
    k

fn suffix_less<T>(s: List<T>, a: Int, b: Int) -> Bool:
    let k = common_prefix(s, a, b)
    if b + k == s.len: return false
    a + k == s.len or s[a + k] < s[b + k]

@test
fn z_and_prefix_function_match_their_definitions():
    assert z_function("aaabaab") == [7, 2, 1, 0, 2, 1, 0]
    assert prefix_function("abacaba") == [0, 0, 1, 0, 1, 2, 3]
    assert z_function("") == [] and prefix_function("") == []
    for _ in 0..40:
        let s = random_word(rand(1, 14), 2)
        let n = s.len
        let z = z_function(s)
        let p = prefix_function(s)
        for i in 0..n:
            assert z[i] == common_prefix(s, 0, i)
            assert p[i] == max(k for k in 0..=i if s[0..k] == s[i + 1 - k..i + 1])
        assert period(s) == min(d for d in 1..=n if s[0..n - d] == s[d..n])

@test
fn find_all_matches_a_scan():
    assert [i for i in find_all("abababa", "aba")] == [0, 2, 4]
    assert [i for i in find_all("abc", "")] == [0, 1, 2, 3]
    assert [i for i in find_all("ab", "abc")] == []
    for _ in 0..40:
        let text = random_word(rand(0, 20), 2)
        let pat = random_word(rand(1, 4), 2)
        let want = [i for i in 0..=text.len - pat.len if text[i..i + pat.len] == pat]
        assert [i for i in find_all(text, pat)] == want
    // any list of comparable values, not only characters
    assert [i for i in find_all([1, 2, 1, 2, 1], [1, 2, 1])] == [0, 2]

@test
fn suffix_array_and_lcp_match_sorting_the_suffixes():
    assert suffix_array("banana") == [5, 3, 1, 0, 4, 2]
    assert lcp_array("banana", suffix_array("banana")) == [1, 3, 0, 0, 2]
    assert suffix_array("") == [] and suffix_array("x") == [0] and lcp_array("x", [0]) == []
    for _ in 0..40:
        let s = random_word(rand(1, 18), rand(1, 3))
        let sa = suffix_array(s)
        var want = [i for i in 0..s.len]
        sort(mut want, by: i => [s[j] for j in i..s.len])
        assert sa == want
        let lcp = lcp_array(s, sa)
        for j in 0..s.len - 1: assert lcp[j] == common_prefix(s, sa[j], sa[j + 1])

@test
fn rolling_hash_over_str_list_and_domain_arrays():
    // 1. Str substring hashes match brute-force equality
    for _ in 0..20:
        let s = Str(random_word(rand(1, 16), 3))
        let h = RollingHash(s)
        assert h.len == s.len
        for l in 0..s.len:
            for r in l..=s.len:
                let hv = h.get(l..r)
                assert hv == h.slice(l, r)
                if l < r: assert hv == h.get(l..=r - 1)
        for _ in 0..30:
            let len = rand(1, s.len)
            let a = rand(0, s.len - len)
            let b = rand(0, s.len - len)
            assert (h.get(a..a + len) == h.get(b..b + len)) == (s[a..a + len] == s[b..b + len])
        // combine matches full range for every split
        for mid in 0..=s.len:
            assert h.combine(h.get(..mid), h.get(mid..s.len), s.len - mid) == h.all
    // 2. Domain-indexed [D]Int array (including negative values and zeros)
    domain D = 1..=8
    let arr = [D] i => [0, 0, -5, 7, 0, 0, -5, 7][i.to_idx]
    let hd = RollingHash(arr)
    assert hd.get(D(1)..=D(4)) == hd.get(D(5)..=D(8))
    assert hd.get(D(1)..=D(1)) != hd.get(D(1)..=D(2))   // [0] vs [0, 0] never collide
    assert hd.get(D(3)..D(3)) == HashVal(0, 0)
    assert hd.get(..D(5)) == hd.get(D(5)..)
    // 3. Shared bases compare across two distinct sequences
    let h1 = RollingHash("abacaba")
    let h2 = RollingHash.with_bases("xxabayy", h1.bases)
    assert h1.get(0..3) == h2.get(2..5) and h1.get(4..7) == h2.get(2..5)
    assert h1.get(0..3) != h2.get(0..3)
    // 4. Nested 2D hashing: [R]HashVal fed into a second RollingHash with independent salted bases
    domain R = 0..3
    domain C = 0..3
    let g = [R, C] (r, c) => [[1, 2, 3], [4, 5, 6], [7, 8, 9]][r.to_idx][c.to_idx]
    let gt = [C, R] (c, r) => g[r, c]
    let row_b = RollingHash([g[R(0), c] for c in C]).bases
    let row_hashes = [R] r => RollingHash.with_bases([g[r, c] for c in C], row_b).all
    let col_hashes = [C] c => RollingHash.with_bases([gt[c, r] for r in R], row_b).all
    let h_grid = RollingHash(row_hashes)
    let h_trans = RollingHash.with_bases(col_hashes, h_grid.bases)
    assert h_grid.bases != row_b
    assert h_grid.all != h_trans.all
    // Same matrix hashes equal under matching row & outer bases
    let row_hashes2 = [R] r => RollingHash.with_bases([g[r, c] for c in C], row_b).all
    assert RollingHash.with_bases(row_hashes2, h_grid.bases).all == h_grid.all

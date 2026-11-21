//! String algorithms. They only compare elements, so they work on a `Str`
//! (ASCII bytes) and on any `List<T>` alike. Positions are 0-based.

use std.seq.{reverse}

/// `s` written `n` times over: `"0".repeat(4)` is `"0000"`.
pub fn repeat(self: Str, n: Int) -> Str:
    assert n >= 0, "`repeat` needs a count of 0 or more"
    Str(self[i % self.len] for i in 0..n * self.len)

/// True when `s` opens with `p`: `"contest".starts_with("con")`. O(p.len).
pub fn starts_with(self: Str, p: Str) -> Bool = p.len <= self.len and all(
    self[i] == p[i] for i in 0..p.len
)

/// True when `s` closes with `p`: `"contest".ends_with("test")`. O(p.len).
pub fn ends_with(self: Str, p: Str) -> Bool:
    let d = self.len - p.len
    d >= 0 and all(self[d + i] == p[i] for i in 0..p.len)

/// Where `p` first stands in `s`, or `none`: `"banana".find("nan")` is `2`. `p` is a `Str` or
/// one `Char`: `"banana".find('n')` is `2`. O(s.len + p.len).
pub fn find<P>(self: Str, p: P) -> ?Int:
    let pat = "" + p
    for i in find_all([c for c in self], [c for c in pat]): return i
    none

/// How many times `c` stands in `s`: `"banana".count('a')` is `3`. O(s.len).
pub fn count(self: Str, c: Char) -> Int:
    var n = 0
    for x in self:
        if x == c: n += 1
    n

/// The last index where `c` stands in `s`, or `none`: `"banana".rfind('a')` is `some(5)`.
/// O(s.len).
pub fn rfind(self: Str, c: Char) -> ?Int:
    for i in (0..self.len).rev:
        if self[i] == c: return some(i)
    none

/// The small letter of a capital, and any other character as it is: `'A'.lower` is `'a'`.
pub @get fn lower(self: Char) -> Char = if 'A' <= self and self <= 'Z': Char(self.ord + 32) else: self

/// The capital of a small letter, and any other character as it is: `'a'.upper` is `'A'`.
pub @get fn upper(self: Char) -> Char = if 'a' <= self and self <= 'z': Char(self.ord - 32) else: self

/// The value of a decimal digit: `'7'.digit` is 7. Stops on any other character.
pub @get fn digit(self: Char) -> Int:
    assert '0' <= self and self <= '9', "`digit` needs one of '0' to '9'"
    self.ord - '0'.ord

/// `z[i]` is the length of the longest common prefix of `s` and `s[i..]`; `z[0]` is `s.len`. O(n).
@allow(unproven_list_index)
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
@allow(unproven_list_index)
pub fn prefix_function<T>(s: List<T>) -> List<Int>:
    var p = List.filled(s.len, 0)
    for i in 1..s.len:
        var k = p[i - 1]
        while k > 0 and s[i] != s[k]: k = p[k - 1]
        if s[i] == s[k]: k += 1
        p[i] = k
    p

/// The start of every occurrence of `pat` in `self`: `s.find_all(p)`, in increasing order (Knuth-Morris-Pratt).
/// O(text.len + pat.len). An empty pattern occurs at every position, including the end.
@allow(unproven_list_index)
pub fn find_all<T>(self: List<T>, pat: List<T>) -> yields Int:
    if pat.is_empty:
        for i in 0..=self.len: yield i
        return
    let p = prefix_function(pat)
    var k = 0
    for i in 0..self.len:
        while k > 0 and self[i] != pat[k]: k = p[k - 1]
        if self[i] == pat[k]: k += 1
        if k == pat.len:
            yield i + 1 - k
            k = p[k - 1]

/// The length of the shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` for all `i`. O(n).
pub fn period<T>(s: List<T>) -> Int = if s.is_empty: 0 else: s.len - prefix_function(s).last

fn induced_sort(
    vec: List<Int>,
    counts: List<Int>,
    is_l: List<Bool>,
    lms: List<Int>,
    mut sa: List<Int>,
):
    let n = vec.len
    let alpha = counts.len
    var bkt = List.filled(alpha, 0)
    var sum = 0
    for c in 0..alpha:
        sum += counts[c]
        bkt[c] = sum
    for i in 0..n: sa[i] = -1
    for k in (0..lms.len).rev:
        let pos = lms[k]
        let c = vec[pos]
        bkt[c] -= 1
        sa[bkt[c]] = pos
    sum = 0
    for c in 0..alpha:
        bkt[c] = sum
        sum += counts[c]
    for k in 0..n:
        let p = sa[k] - 1
        if p >= 0 and is_l[p]:
            let c = vec[p]
            sa[bkt[c]] = p
            bkt[c] += 1
    sum = 0
    for c in 0..alpha:
        sum += counts[c]
        bkt[c] = sum
    for k in (1..n).rev:
        let p = sa[k] - 1
        if p >= 0 and not is_l[p]:
            let c = vec[p]
            bkt[c] -= 1
            sa[bkt[c]] = p

@allow(unproven_list_index)
fn sa_is(vec: List<Int>, alpha: Int) -> List<Int>:
    let n = vec.len
    var sa = List.filled(n, 0)
    if n <= 1: return sa
    var counts = List.filled(alpha, 0)
    for c in vec: counts[c] += 1
    var is_l = List.filled(n, false)
    var lms = List<Int>()
    for i in (0..n - 1).rev:
        is_l[i] = vec[i] > vec[i + 1] or (vec[i] == vec[i + 1] and is_l[i + 1])
        if is_l[i] and not is_l[i + 1]: lms.push(i + 1)
    reverse(mut lms)
    induced_sort(vec, counts, is_l, lms, mut sa)
    let m = lms.len
    var sorted_lms = List.filled(m, 0)
    var k_lms = 0
    for x in sa:
        if x >= 1 and not is_l[x] and is_l[x - 1]:
            sorted_lms[k_lms] = x
            k_lms += 1
    var name = 0
    sa[n - 1] = 0
    for k in 1..m:
        let i = sorted_lms[k - 1]
        let j = sorted_lms[k]
        var diff = vec[i] != vec[j]
        if not diff:
            var (a, b) = (i + 1, j + 1)
            while true:
                if vec[a] != vec[b]:
                    diff = true
                    break
                let la = not is_l[a] and is_l[a - 1]
                let lb = not is_l[b] and is_l[b - 1]
                if la or lb:
                    diff = not (la and lb)
                    break
                a += 1
                b += 1
        if diff: name += 1
        sa[j] = name
    if name + 1 < m:
        var lms_vec = List.filled(m, 0)
        for i in 0..m: lms_vec[i] = sa[lms[i]]
        let lms_sa = sa_is(lms_vec, name + 1)
        for i in 0..m: sorted_lms[i] = lms[lms_sa[i]]
    induced_sort(vec, counts, is_l, sorted_lms, mut sa)
    sa

/// The starts of the suffixes of `s` in sorted order (SA-IS).
/// O(n) for bounded alphabets (<= 256 distinct values) and O(n log n) for general `T`.
@allow(unproven_list_index)
pub fn suffix_array<T>(s: List<T>) -> List<Int>:
    let n = s.len
    if n <= 1: return [i for i in 0..n]
    var vec = List.filled(n + 1, 0)
    var alpha = 0
    // Fast path for bounded alphabets (e.g. ASCII/Str): avoid O(n log n) sort.
    var uniq = List<T>()
    var ids = List<Int>()
    var small = true
    for i in 0..n:
        let x = s[i]
        var (lo, hi) = (0, uniq.len)
        while lo < hi:
            let mid = (lo + hi) >> 1
            if uniq[mid] < x: lo = mid + 1 else: hi = mid
        if lo < uniq.len and uniq[lo] == x:
            vec[i] = ids[lo]
        elif uniq.len == 256:
            small = false
            break
        else:
            let id = uniq.len
            uniq.push(x)
            ids.push(id)
            for j in (lo + 1..uniq.len).rev:
                uniq[j] = uniq[j - 1]
                ids[j] = ids[j - 1]
            uniq[lo] = x
            ids[lo] = id
            vec[i] = id
    if small:
        alpha = uniq.len
        if alpha == 1: return [n - 1 - i for i in 0..n]
        var remap = List.filled(alpha, 0)
        for r in 0..alpha: remap[ids[r]] = r + 1
        for i in 0..n: vec[i] = remap[vec[i]]
    else:
        var order = [i for i in 0..n]
        sort(mut order, by: i => s[i])
        alpha = 1
        vec[order[0]] = 1
        for j in 1..n:
            if s[order[j - 1]] < s[order[j]]: alpha += 1
            vec[order[j]] = alpha
        if alpha == n: return order
    var sa = sa_is(vec, alpha + 1)
    for i in 0..n: sa[i] = sa[i + 1]
    sa.pop()
    sa

/// `lcp[j]` is the length of the longest common prefix of the suffixes at `sa[j]` and `sa[j + 1]`
/// (Kasai). O(n); the result has `n - 1` entries.
@allow(unproven_list_index)
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
    pub fn with_bases<S>(
        seq: S,
        bases: (Int, Int),
    ) -> RollingHash = RollingHash.new(seq, bases: bases)

    /// Number of hashed elements.
    pub @get fn len(self) -> Int = p1.len - 1

    /// Hash of the entire sequence. O(1).
    pub @get fn all(self) -> HashVal = self.slice(0, p1.len - 1)

    /// Hash of the half-open 0-based slice `[l, r)`; `HashVal(0, 0)` when `l == r`. Stops on
    /// `l > r` or an end outside the sequence. O(1).
    pub fn slice(self, l: Int, r: Int) -> HashVal:
        assert 0 <= l and l <= r and r <= p1.len - 1, "`slice(l, r)` needs `0 <= l <= r <= len`"
        if l == r: return HashVal(0, 0)
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

fn random_word(n: Int, letters: Int) -> List<Char> = [
    Char(Int('a') + rand(0, letters - 1)) for _ in 0..n
]

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
    assert [i for i in "abababa".find_all("aba")] == [0, 2, 4]
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
    assert suffix_array("aaaaa") == [4, 3, 2, 1, 0]
    assert suffix_array("abcdef") == [0, 1, 2, 3, 4, 5]
    assert suffix_array("fedcba") == [5, 4, 3, 2, 1, 0]
    for _ in 0..40:
        let s = random_word(rand(1, 18), rand(1, 3))
        let sa = suffix_array(s)
        var want = [i for i in 0..s.len]
        sort(mut want, by: i => [s[j] for j in i..s.len])
        assert sa == want
        let lcp = lcp_array(s, sa)
        for j in 0..s.len - 1: assert lcp[j] == common_prefix(s, sa[j], sa[j + 1])
    // Large alphabet (>256 distinct values) and negative ints exercise the fallback path
    let big_uniq = [300 - i for i in 0..270]
    assert suffix_array(big_uniq) == [269 - i for i in 0..270]
    var big_dup = [i % 260 - 130 for i in 0..280]
    big_dup.push(-130)
    big_dup.push(-129)
    let sa_big = suffix_array(big_dup)
    assert sa_big.len == big_dup.len
    for j in 0..sa_big.len - 1: assert suffix_less(big_dup, sa_big[j], sa_big[j + 1])

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
    assert hd.get(D(1)..=D(1)) != hd.get(D(1)..=D(2))  // [0] vs [0, 0] never collide
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

@test
fn repeat_writes_a_string_over():
    assert "ab".repeat(3) == "ababab"
    assert "0".repeat(4) == "0000"
    assert "ab".repeat(0) == "" and "".repeat(5) == ""

@test
fn starts_ends_and_find_on_text():
    let s = "Banana"
    assert s.starts_with("Ban") and not s.starts_with("an") and s.starts_with("")
    assert s.ends_with("ana") and not s.ends_with("Bananas") and s.ends_with("")
    assert s.find("nan") == 2 and s.find("x") == none and s.find("") == 0

@test
fn lower_and_upper_change_only_letters():
    assert Str(c.lower for c in "aZ-9") == "az-9"
    assert Str(c.upper for c in "aZ-9") == "AZ-9"

@test
fn digit_is_the_value_of_a_decimal_character():
    assert '0'.digit == 0 and '7'.digit == 7 and sum(c.digit for c in "409") == 13

@test
fn count_counts_one_character():
    assert "banana".count('a') == 3 and "banana".count('z') == 0 and "".count('a') == 0

@test
fn rfind_gives_the_last_place():
    assert "banana".rfind('a') == some(5)
    assert "banana".rfind('b') == some(0)
    assert "banana".rfind('x') == none
    assert "".rfind('a') == none

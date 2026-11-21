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
    for i in find_all(self, pat): return i
    none

/// How many times `c` stands in `s`: `"banana".count('a')` is `3`. O(s.len).
pub fn count(self: Str, c: Char) -> Int:
    var n = 0
    for x in self:
        if x == c: n += 1
    n

/// Where `p` last stands in `s`, or `none`: `"banana".rfind("an")` is `3`. `p` is a `Str` or
/// one `Char`: `"banana".rfind('a')` is `5`. O(s.len + p.len).
pub fn rfind<P>(self: Str, p: P) -> ?Int:
    let pat = "" + p
    var last: ?Int = none
    for i in find_all(self, pat): last = i
    last

/// The small letter of a capital, and any other character as it is: `'A'.lower` is `'a'`.
pub @get fn lower(self: Char) -> Char = if 'A' <= self and self <= 'Z': Char(self.ord + 32) else: self

/// The capital of a small letter, and any other character as it is: `'a'.upper` is `'A'`.
pub @get fn upper(self: Char) -> Char = if 'a' <= self and self <= 'z': Char(self.ord - 32) else: self

/// The value of a decimal digit: `'7'.digit` is 7. Stops on any other character.
pub @get fn digit(self: Char) -> Int:
    assert '0' <= self and self <= '9', "`digit` needs one of '0' to '9'"
    self.ord - '0'.ord

/// `z[i]` is the length of the longest common prefix of `s` and `s[i..]`; `z[0]` is `s.len`.
/// Works on any `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion. O(n).
@allow(unproven_list_index)
pub fn z_function<S: Indexed>(s: S) -> List<Int>:
    let n = s.len
    var z = List.filled(n, 0)
    if n == 0: return z
    z[0] = n
    var (l, r) = (0, 0)
    for i in 1..n:
        var k = if i < r: min(r - i, z[i - l]) else: 0
        while i + k < n and s.at(k) == s.at(i + k): k += 1
        z[i] = k
        if i + k > r: (l, r) = (i, i + k)
    z

/// `p[i]` is the length of the longest proper prefix of `s[..=i]` that is also its suffix.
/// Works on any `Indexed` sequence (`Str`, `List<T>`, `[D]T`, `Range<I>`, views) without conversion. O(n).
@allow(unproven_list_index)
pub fn prefix_function<S: Indexed>(s: S) -> List<Int>:
    var p = List.filled(s.len, 0)
    for i in 1..s.len:
        var k = p[i - 1]
        while k > 0 and s.at(i) != s.at(k): k = p[k - 1]
        if s.at(i) == s.at(k): k += 1
        p[i] = k
    p

/// The start of every occurrence of `pat` in `self`: `s.find_all(p)`, in increasing order (Knuth-Morris-Pratt).
/// Works on any `Indexed` sequences (`Str`, `List<T>`, `[D]T`, views) without conversion.
/// O(text.len + pat.len). An empty pattern occurs at every position, including the end.
@allow(unproven_list_index)
pub fn find_all<S: Indexed, P: Indexed>(self: S, pat: P) -> yields Int:
    if pat.is_empty:
        for i in 0..=self.len: yield i
        return
    let p = prefix_function(pat)
    var k = 0
    for i in 0..self.len:
        while k > 0 and self.at(i) != pat.at(k): k = p[k - 1]
        if self.at(i) == pat.at(k): k += 1
        if k == pat.len:
            yield i + 1 - k
            k = p[k - 1]

/// Length of the longest common prefix of `self` and `other`. Works on any `Indexed` sequences. O(min(|self|, |other|)).
@allow(unproven_list_index)
pub fn common_prefix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int:
    let lim = min(self.len, other.len)
    var i = 0
    while i < lim and self.at(i) == other.at(i): i += 1
    i

/// Length of the longest common suffix of `self` and `other`. Works on any `Indexed` sequences. O(min(|self|, |other|)).
@allow(unproven_list_index)
pub fn common_suffix_len<A: Indexed, B: Indexed>(self: A, other: B) -> Int:
    let (na, nb) = (self.len, other.len)
    let lim = min(na, nb)
    var i = 0
    while i < lim and self.at(na - 1 - i) == other.at(nb - 1 - i): i += 1
    i

/// The length of the shortest period of `s`: the smallest `d >= 1` with `s[i] == s[i + d]` for all `i`. O(n).
pub fn period<S: Indexed>(s: S) -> Int = if s.is_empty: 0 else: s.len - prefix_function(s).last

/// True when `s` reads the same forwards and backwards. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn is_palindrome<S: Indexed>(s: S) -> Bool:
    let n = s.len
    for i in 0..n / 2:
        if s.at(i) != s.at(n - 1 - i): return false
    true

/// All non-empty proper border lengths of `s`, longest first. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn borders<S: Indexed>(s: S) -> yields Int:
    if s.is_empty: return
    let pi = prefix_function(s)
    var k = pi[s.len - 1]
    while k > 0:
        yield k
        k = pi[k - 1]

/// `res[i]` is the length of the longest prefix of `pat` ending at `self[i]` (KMP).
/// Works on any `Indexed` sequences. O(|self| + |pat|).
@allow(unproven_list_index)
pub fn prefix_matches<S: Indexed, P: Indexed>(self: S, pat: P) -> List<Int>:
    let n = self.len
    let m = pat.len
    var out = List.filled(n, 0)
    if m == 0: return out
    let pi = prefix_function(pat)
    var k = 0
    for i in 0..n:
        while k > 0 and (k == m or self.at(i) != pat.at(k)): k = pi[k - 1]
        if self.at(i) == pat.at(k): k += 1
        out[i] = k
    out

/// Length of the longest suffix of `a` that is a prefix of `b` (at most `min(|a|, |b|)`).
/// Works on any `Indexed` sequences. O(|a| + |b|).
@allow(unproven_list_index)
pub fn overlap<A: Indexed, B: Indexed>(a: A, b: B) -> Int:
    let keep = min(a.len, b.len)
    if keep == 0: return 0
    let pi = prefix_function([b.at(i) for i in 0..keep])
    var k = 0
    for i in a.len - keep..a.len:
        while k > 0 and (k == keep or a.at(i) != b.at(k)): k = pi[k - 1]
        if a.at(i) == b.at(k): k += 1
    k

/// Start indices of the unique non-increasing Lyndon factorization of `s` (Duval).
/// Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn lyndon_factorization<S: Indexed>(s: S) -> List<Int>:
    let n = s.len
    var starts: List<Int> = []
    var i = 0
    unchecked:
        while i < n:
            var j = i + 1
            var k = i
            while j < n and s.at(k) <= s.at(j):
                if s.at(k) < s.at(j): k = i else: k += 1
                j += 1
            let step = j - k
            while i <= k:
                starts.push(i)
                i += step
    starts

/// Start index of the lexicographically least cyclic rotation of `s` (Duval/Booth).
/// Ties go to the smallest index; returns `0` for an empty sequence. Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn min_rotation<S: Indexed>(s: S) -> Int:
    let n = s.len
    if n <= 1: return 0
    var (a, b) = (0, 1)
    var k = 0
    unchecked:
        while a < n and b < n and k < n:
            let ai = a + k
            let bi = b + k
            let x = s.at(if ai < n: ai else: ai - n)
            let y = s.at(if bi < n: bi else: bi - n)
            if x == y:
                k += 1
            else:
                if x > y: a += k + 1 else: b += k + 1
                if a == b: b += 1
                k = 0
    min(a, b)

/// Manacher palindrome radii. `odd[i]` is the number of palindromes centred on `s[i]`;
/// `even[i]` is the number centred between `s[i - 1]` and `s[i]`.
pub struct Palindromes(odd: List<Int>, even: List<Int>):
    /// True when the range `r` (`l..r`, `l..=r`, or `Range<D>`) is a palindrome. O(1).
    pub fn is_palindrome<R>(self, r: R) -> Bool:
        if r.is_empty: return true
        let l = r.start.ord
        let hi = r.end.ord
        let n = odd.len
        assert 0 <= l and l <= hi and hi <= n, "`is_palindrome(r)` needs `0 <= lo <= hi <= len`"
        let m = hi - l
        if m <= 1: return true
        if m % 2 == 1: odd[(l + hi) / 2] * 2 - 1 >= m else: even[(l + hi) / 2] * 2 >= m

    /// Total number of non-empty palindromic substrings. O(n).
    pub @get fn count(self) -> Int = sum(x for x in odd) + sum(x for x in even)

    /// Longest palindromic substring range `(l, r)` (half-open); ties go to the leftmost. O(n).
    pub @get fn longest(self) -> (l: Int, r: Int):
        var (best_l, best_r) = (0, 0)
        unchecked:
            for i in 0..odd.len:
                let olen = odd[i] * 2 - 1
                if olen > best_r - best_l:
                    best_l = i - odd[i] + 1
                    best_r = i + odd[i]
                let elen = even[i] * 2
                if elen > best_r - best_l:
                    best_l = i - even[i]
                    best_r = i + even[i]
        (l: best_l, r: best_r)

/// Linear-time palindrome radii (Manacher's algorithm). Works on any `Indexed` sequence. O(n).
@allow(unproven_list_index)
pub fn manacher<S: Indexed>(s: S) -> Palindromes:
    let n = s.len
    var odd = List.filled(n, 0)
    var even = List.filled(n, 0)
    var (l, r) = (0, -1)
    unchecked:
        for i in 0..n:
            var k = if i > r: 1 else: min(odd[l + r - i], r - i + 1)
            while i - k >= 0 and i + k < n and s.at(i - k) == s.at(i + k): k += 1
            odd[i] = k
            if i + k - 1 > r: (l, r) = (i - k + 1, i + k - 1)
        (l, r) = (0, -1)
        for i in 0..n:
            var k = if i > r: 0 else: min(even[l + r - i + 1], r - i + 1)
            while i - k - 1 >= 0 and i + k < n and s.at(i - k - 1) == s.at(i + k): k += 1
            even[i] = k
            if i + k - 1 > r: (l, r) = (i - k, i + k - 1)
    Palindromes(odd, even)

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
        order.sort(by: i => s[i])
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

/// Suffix array + rank + LCP + sparse-table RMQ for O(1) LCP and substring comparison.
/// Built in O(n log n) time and memory (SA-IS + Kasai + sparse table).
pub struct SuffixArray<T>:
    s: List<T>
    sa: List<Int>
    rank: List<Int>
    lcp: List<Int>
    rmq: List<Int>
    rmq_n: Int

    @allow(unproven_list_index)
    /// Build suffix array and Kasai LCP array for `s`. O(n log n).
    pub fn new(s: List<T>) -> SuffixArray<T>:
        let n = s.len
        let sa = suffix_array(s)
        var rank = List.filled(n, 0)
        for j in 0..n: rank[sa[j]] = j
        let lcp = lcp_array(s, sa)
        let m = lcp.len
        var rmq: List<Int> = []
        if m > 0:
            var lg = 1
            while (1 << lg) <= m: lg += 1
            rmq = List.filled(lg * m, 0)
            for i in 0..m: rmq[i] = lcp[i]
            for b in 1..lg:
                let step = 1 << (b - 1)
                let lim = m - (1 << b) + 1
                let cur = b * m
                let prev = (b - 1) * m
                for i in 0..lim: rmq[cur + i] = min(rmq[prev + i], rmq[prev + i + step])
        SuffixArray(s, sa, rank, lcp, rmq, m)

    /// Length of the indexed sequence. O(1).
    pub @get fn len(self) -> Int = s.len

    @allow(unproven_list_index)
    fn rmq_min(self, lo: Int, hi: Int) -> Int:
        let k = bit_width(hi - lo) - 1
        let base = k * rmq_n
        min(rmq[base + lo], rmq[base + hi - (1 << k)])

    /// Longest common prefix of suffixes `s[i..]` and `s[j..]`. O(1).
    @allow(unproven_list_index)
    pub fn lcp_between(self, i: Int, j: Int) -> Int:
        let n = s.len
        assert 0 <= i and i < n and 0 <= j and j < n, "`lcp_between(i, j)` needs `0 <= i, j < len`"
        if i == j: return n - i
        let a = min(rank[i], rank[j])
        let b = max(rank[i], rank[j])
        self.rmq_min(a, b)

    /// Rank range `lo..hi` in `sa` of all suffixes starting with `pat`. O(|pat| log n).
    @allow(unproven_list_index)
    pub fn matches(self, pat: List<T>) -> Range<Int>:
        let n = s.len
        let m = pat.len
        if m == 0: return 0..n
        var (lo, hi) = (0, n)
        while lo < hi:
            let mid = (lo + hi) >> 1
            let st = sa[mid]
            var k = 0
            while k < m and st + k < n and s[st + k] == pat[k]: k += 1
            let less = if k == m: false elif st + k == n: true else: s[st + k] < pat[k]
            if less: lo = mid + 1 else: hi = mid
        let left = lo
        lo = left
        hi = n
        while lo < hi:
            let mid = (lo + hi) >> 1
            let st = sa[mid]
            var k = 0
            while k < m and st + k < n and s[st + k] == pat[k]: k += 1
            let le = if k == m: true elif st + k == n: true else: s[st + k] < pat[k]
            if le: lo = mid + 1 else: hi = mid
        left..lo

    /// Number of occurrences of `pat` in `s`. O(|pat| log n).
    pub fn count(self, pat: List<T>) -> Int:
        let r = self.matches(pat)
        r.end - r.start

    /// Rank range `lo..hi` in `sa` of all suffixes whose LCP with `s[i..]` is at least `len`.
    /// O(log n).
    @allow(unproven_list_index)
    pub fn lcp_range(self, i: Int, len: Int) -> Range<Int>:
        let n = s.len
        assert 0 <= i and i < n and len >= 0, "`lcp_range(i, len)` needs `0 <= i < len` and `len >= 0`"
        if len == 0: return 0..n
        let p = rank[i]
        if i + len > n: return p..p
        var (lo, hi) = (0, p)
        while lo < hi:
            let mid = (lo + hi) >> 1
            if self.rmq_min(mid, p) >= len: hi = mid else: lo = mid + 1
        let left = lo
        lo = p
        hi = n - 1
        while lo < hi:
            let mid = (lo + hi + 1) >> 1
            if self.rmq_min(p, mid) >= len: lo = mid else: hi = mid - 1
        left..lo + 1

    /// Lexicographic comparison of `s[l1..r1]` and `s[l2..r2]`: returns `-1`, `0`, or `1`. O(1).
    @allow(unproven_list_index)
    pub fn compare(self, l1: Int, r1: Int, l2: Int, r2: Int) -> Int:
        let n = s.len
        assert 0 <= l1 and l1 <= r1 and r1 <= n, "`compare` needs `0 <= l1 <= r1 <= len`"
        assert 0 <= l2 and l2 <= r2 and r2 <= n, "`compare` needs `0 <= l2 <= r2 <= len`"
        let n1 = r1 - l1
        let n2 = r2 - l2
        let m = min(n1, n2)
        if m == 0: return if n1 < n2: -1 elif n1 > n2: 1 else: 0
        let c = self.lcp_between(l1, l2)
        if c >= m: return if n1 < n2: -1 elif n1 > n2: 1 else: 0
        if s[l1 + c] < s[l2 + c]: -1 else: 1

    /// Number of distinct non-empty substrings of `s`. O(n).
    pub @get fn distinct(self) -> Int:
        let n = s.len
        var total = n * (n + 1) / 2
        for x in lcp: total -= x
        total

/// All maximal repetitions (runs) of `s` via Bannai's Lyndon-runs theorem.
/// Each `(l, r, period)` is a half-open range `s[l..r]` with minimal period `period`,
/// length `r - l >= 2 * period`, that cannot be extended left or right with period `period`.
/// Returned sorted by `(l, r)`. O(n log n) time and memory.
@allow(unproven_list_index)
pub fn lyndon_runs<T>(s: List<T>) -> List<(l: Int, r: Int, period: Int)>:
    let n = s.len
    if n < 2: return []
    let sa = SuffixArray.new(s)
    var rev = List<T>()
    for i in (0..n).rev: rev.push(s[i])
    let rsa = SuffixArray.new(rev)
    var raw = List<(l: Int, r: Int, period: Int)>()
    unchecked:
        for rev_ord in [false, true]:
            var st = List<Int>()
            for i in (0..n).rev:
                while not st.is_empty:
                    let top = st[st.len - 1]
                    let common = sa.lcp_between(i, top)
                    if top + common == n: break
                    let less = (s[i + common] < s[top + common]) != rev_ord
                    if less: st.pop() else: break
                let j = if st.is_empty: n else: st[st.len - 1]
                st.push(i)
                let p = j - i
                let left = if i == 0: 0 else: rsa.lcp_between(n - i, n - j)
                let right = if j == n: 0 else: sa.lcp_between(i, j)
                let l = i - left
                let r = j + right
                if r - l >= 2 * p: raw.push((l: l, r: r, period: p))
    raw.sort()
    var out = List<(l: Int, r: Int, period: Int)>()
    for run in raw:
        if out.is_empty or out[out.len - 1].l != run.l or out[out.len - 1].r != run.r:
            out.push(run)
    out

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

    /// Build a double-modulus polynomial rolling hash over `seq`. O(n).
    @allow(unproven_list_index)
    pub fn new<S: Seq>(seq: S, bases: ?(Int, Int) = none) -> RollingHash:
        let (b1, b2) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        let n = seq.len
        var p1 = List.filled(n + 1, 0)
        var p2 = List.filled(n + 1, 0)
        var pow1 = List.filled(n + 1, 1)
        var pow2 = List.filled(n + 1, 1)
        var (h1, h2, w1, w2) = (0, 0, 1, 1)
        var idx = 0
        for x in seq:
            idx += 1
            let t = x.hash_token
            let x1 = if 0 <= t and t < 1_000_000_006: t + 1 else: t % 1_000_000_006 + 1
            let x2 = if 0 <= t and t < 1_000_000_008: t + 1 else: t % 1_000_000_008 + 1
            h1 = (h1 * b1 + x1) % 1_000_000_007
            h2 = (h2 * b2 + x2) % 1_000_000_009
            w1 = w1 * b1 % 1_000_000_007
            w2 = w2 * b2 % 1_000_000_009
            unchecked:
                p1[idx] = h1
                p2[idx] = h2
                pow1[idx] = w1
                pow2[idx] = w2
        RollingHash((b1, b2), p1, p2, pow1, pow2)

    /// Build a `RollingHash` reusing another instance's salted `bases` so their hashes compare directly.
    pub fn with_bases<S: Seq>(
        seq: S,
        bases: (Int, Int),
    ) -> RollingHash = RollingHash.new(seq, bases: bases)

    /// Number of hashed elements.
    pub @get fn len(self) -> Int = p1.len - 1

    /// Hash of the entire sequence. O(1).
    pub @get fn all(self) -> HashVal = self.slice(0, p1.len - 1)

    @allow(unproven_list_index)
    fn raw_slice(self, l: Int, r: Int) -> HashVal:
        let d = r - l
        unchecked:
            return HashVal(
                (p1[r] - p1[l] * pow1[d]) % 1_000_000_007,
                (p2[r] - p2[l] * pow2[d]) % 1_000_000_009,
            )
        HashVal(0, 0)

    @allow(unproven_list_index)
    fn eq_sub(self, a: Int, b: Int, len: Int) -> Bool:
        unchecked:
            if ((p1[a + len] - p1[b + len]) - (p1[a] - p1[b]) * pow1[len]) % 1_000_000_007 != 0:
                return false
            return ((p2[a + len] - p2[b + len]) - (p2[a] - p2[b]) * pow2[len]) % 1_000_000_009 == 0
        false

    /// Hash of the half-open 0-based slice `[l, r)`; `HashVal(0, 0)` when `l == r`. Stops on
    /// `l > r` or an end outside the sequence. O(1).
    pub fn slice(self, l: Int, r: Int) -> HashVal:
        assert 0 <= l and l <= r and r <= p1.len - 1, "`slice(l, r)` needs `0 <= l <= r <= len`"
        if l == r: return HashVal(0, 0)
        self.raw_slice(l, r)

    /// Hash of a range (`Range<Int>` such as `l..r`, `l..=r`, `..k`, or `Range<D>` on `[D]T`). O(1).
    pub fn get<R>(self, r: R) -> HashVal:
        if r.is_empty: return HashVal(0, 0)
        self.slice(r.start.ord, r.end.ord)

    /// Concatenate hashes `a` and `b` where `b` has length `b_len`, in O(1).
    pub fn combine(self, a: HashVal, b: HashVal, b_len: Int) -> HashVal:
        HashVal(
            (a.h1 * pow1[b_len] + b.h1) % 1_000_000_007,
            (a.h2 * pow2[b_len] + b.h2) % 1_000_000_009,
        )

    /// Longest common prefix of suffixes starting at `a` and `b`. O(log n).
    pub fn lcp(self, a: Int, b: Int) -> Int:
        let n = p1.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "`lcp(a, b)` needs `0 <= a, b <= len`"
        if a == b: return n - a
        var (lo, hi) = (0, min(n - a, n - b))
        while lo < hi:
            let mid = (lo + hi + 1) >> 1
            if self.eq_sub(a, b, mid): lo = mid else: hi = mid - 1
        lo

    /// Longest common suffix of prefixes `s[..a]` and `s[..b]`. O(log n).
    pub fn lcs(self, a: Int, b: Int) -> Int:
        let n = p1.len - 1
        assert 0 <= a and a <= n and 0 <= b and b <= n, "`lcs(a, b)` needs `0 <= a, b <= len`"
        if a == b: return a
        var (lo, hi) = (0, min(a, b))
        while lo < hi:
            let mid = (lo + hi + 1) >> 1
            if self.eq_sub(a - mid, b - mid, mid): lo = mid else: hi = mid - 1
        lo

    /// Longest common prefix of `self[a..]` and `other[b..]`. Requires `self.bases == other.bases`.
    /// O(log min(|self|, |other|)).
    pub fn lcp_with(self, a: Int, other: RollingHash, b: Int) -> Int:
        assert bases == other.bases, "`lcp_with` needs matching `bases`"
        let n1 = p1.len - 1
        let n2 = other.len
        assert 0 <= a and a <= n1 and 0 <= b and b <= n2, "`lcp_with` indices out of range"
        var (lo, hi) = (0, min(n1 - a, n2 - b))
        while lo < hi:
            let mid = (lo + hi + 1) >> 1
            if self.raw_slice(a, a + mid) == other.raw_slice(
                b,
                b + mid,
            ): lo = mid else: hi = mid - 1
        lo

/// Double-mod 2D polynomial prefix hash over a grid with row domain `R` and column domain `C`.
/// Built in O(R * C); `rect(rows, cols)` is O(1) and returns a `HashVal`.
pub struct Hash2D<R, C>(bases: (Int, Int)):
    pre1: List<List<Int>>
    pre2: List<List<Int>>
    pow_r1: List<Int>
    pow_c1: List<Int>
    pow_r2: List<Int>
    pow_c2: List<Int>

    @allow(unproven_list_index)
    /// Build a 2D polynomial rolling hash over grid `(R, C)`. O(|R| * |C|).
    pub fn new(code: (R, C) -> Int, bases: ?(Int, Int) = none) -> Hash2D<R, C>:
        let (br1, bc1) = bases ?? (rand(1 << 20, 1 << 29), rand(1 << 20, 1 << 29))
        let br2 = (br1 ^ 324508639) % (RH_M2 - 2) + 2
        let bc2 = (bc1 ^ 715827883) % (RH_M2 - 2) + 2
        let rows = R.K
        let cols = C.K
        var pre1 = [[0 for _ in 0..=cols] for _ in 0..=rows]
        var pre2 = [[0 for _ in 0..=cols] for _ in 0..=rows]
        for r in R:
            let i = r.to_idx
            for c in C:
                let j = c.to_idx
                let v = code(r, c)
                let v1 = v % (RH_M1 - 1) + 1
                let v2 = v % (RH_M2 - 1) + 1
                let a1 = (pre1[i][j + 1] * br1 + pre1[i + 1][j] * bc1) % RH_M1
                let b1 = pre1[i][j] * br1 % RH_M1 * bc1 % RH_M1
                pre1[i + 1][j + 1] = (a1 - b1 + v1) % RH_M1
                let a2 = (pre2[i][j + 1] * br2 + pre2[i + 1][j] * bc2) % RH_M2
                let b2 = pre2[i][j] * br2 % RH_M2 * bc2 % RH_M2
                pre2[i + 1][j + 1] = (a2 - b2 + v2) % RH_M2
        var pow_r1 = [1]
        var pow_r2 = [1]
        for _ in 0..rows:
            pow_r1.push(pow_r1[pow_r1.len - 1] * br1 % RH_M1)
            pow_r2.push(pow_r2[pow_r2.len - 1] * br2 % RH_M2)
        var pow_c1 = [1]
        var pow_c2 = [1]
        for _ in 0..cols:
            pow_c1.push(pow_c1[pow_c1.len - 1] * bc1 % RH_M1)
            pow_c2.push(pow_c2[pow_c2.len - 1] * bc2 % RH_M2)
        Hash2D((br1, bc1), pre1, pre2, pow_r1, pow_c1, pow_r2, pow_c2)

    /// Hash of the axis-aligned rectangle `rows` x `cols`. O(1).
    @allow(unproven_list_index)
    pub fn rect(self, rows: Range<R>, cols: Range<C>) -> HashVal:
        if rows.is_empty or cols.is_empty: return HashVal(0, 0)
        let (r1, r2, c1, c2) = (rows.start.ord, rows.end.ord, cols.start.ord, cols.end.ord)
        let hr1 = pow_r1[r2 - r1]
        let hc1 = pow_c1[c2 - c1]
        let h1 = (
            pre1[r2][c2] - pre1[r1][c2] * hr1 - pre1[r2][c1] * hc1 + pre1[r1][c1] * hr1 % RH_M1 * hc1
        ) % RH_M1
        let hr2 = pow_r2[r2 - r1]
        let hc2 = pow_c2[c2 - c1]
        let h2 = (
            pre2[r2][c2] - pre2[r1][c2] * hr2 - pre2[r2][c1] * hc2 + pre2[r1][c1] * hr2 % RH_M2 * hc2
        ) % RH_M2
        HashVal(h1, h2)

// ---- alphabet-domain automata

/// Online suffix automaton over alphabet domain `A`.
/// O(n * |A|) flat memory, O(n) amortized build, O(states) `topo_order` and `occurrences`.
pub struct SuffixAutomaton<A>:
    alpha: Int
    next: List<Int>
    links: List<Int>
    max_len: List<Int>
    first_end: List<Int>
    prefix: List<Bool>
    last: Int

    /// Create an empty suffix automaton over alphabet `A`. O(1).
    pub fn new() -> SuffixAutomaton<A>:
        let k = A.K
        SuffixAutomaton(k, List.filled(k, -1), [-1], [0], [-1], [false], 0)

    /// Build the suffix automaton of `s`. O(|s| * |A|) memory, O(|s|) amortized time.
    pub fn of(s: List<A>) -> SuffixAutomaton<A>:
        var sam = SuffixAutomaton<A>.new()
        for a in s: sam.push(a)
        sam

    @allow(unproven_list_index)
    fn add_empty_state(mut self, length: Int, is_prefix: Bool) -> Int:
        let id = links.len
        for _ in 0..alpha: next.push(-1)
        links.push(-1)
        max_len.push(length)
        first_end.push(length - 1)
        prefix.push(is_prefix)
        id

    @allow(unproven_list_index)
    fn clone_state(mut self, src: Int, length: Int) -> Int:
        let id = links.len
        let base = src * alpha
        for j in 0..alpha: next.push(next[base + j])
        links.push(links[src])
        max_len.push(length)
        first_end.push(first_end[src])
        prefix.push(false)
        id

    /// Append one letter `a`. O(1) amortized.
    @allow(unproven_list_index)
    pub fn push(mut self, a: A):
        let c = a.to_idx
        let cur = self.add_empty_state(max_len[last] + 1, true)
        unchecked:
            var p = last
            while p != -1 and next[p * alpha + c] == -1:
                next[p * alpha + c] = cur
                p = links[p]
            if p == -1:
                links[cur] = 0
            else:
                let q = next[p * alpha + c]
                if max_len[q] == max_len[p] + 1:
                    links[cur] = q
                else:
                    let clone = self.clone_state(q, max_len[p] + 1)
                    links[q] = clone
                    links[cur] = clone
                    while p != -1 and next[p * alpha + c] == q:
                        next[p * alpha + c] = clone
                        p = links[p]
            last = cur

    /// Root state (empty string): always `0`. O(1).
    pub @get fn root(self) -> Int = 0

    /// Number of states. O(1).
    pub @get fn len(self) -> Int = links.len

    /// Transition from `state` on `a`, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> ?Int:
        let t = next[state * alpha + a.to_idx]
        if t == -1: none else: t

    /// Suffix link of `state`, or `none` at the root. O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> ?Int:
        let p = links[state]
        if p == -1: none else: p

    /// Length of the longest substring in `state`. O(1).
    @allow(unproven_list_index)
    pub fn longest(self, state: Int) -> Int = max_len[state]

    /// Length of the shortest substring in `state` (`0` at the root). O(1).
    @allow(unproven_list_index)
    pub fn shortest(
        self,
        state: Int,
    ) -> Int = if links[state] == -1: 0 else: max_len[links[state]] + 1

    /// 0-based right endpoint of the first occurrence of `state` (`-1` at the root). O(1).
    @allow(unproven_list_index)
    pub fn first_pos(self, state: Int) -> Int = first_end[state]

    /// State of `word`, or `none` if `word` is not a substring. O(|word|).
    @allow(unproven_list_index)
    pub fn find(self, word: List<A>) -> ?Int:
        var at = 0
        for a in word:
            let t = next[at * alpha + a.to_idx]
            if t == -1: return none
            at = t
        at

    /// True when `word` is a substring of the indexed string. O(|word|).
    pub fn contains(self, word: List<A>) -> Bool = self.find(word) != none

    /// Number of distinct non-empty substrings. O(states).
    pub @get fn distinct(self) -> Int = sum(max_len[v] - max_len[links[v]] for v in 1..links.len)

    /// All states sorted by non-decreasing `longest(state)` via counting sort. O(states).
    @allow(unproven_list_index)
    pub fn topo_order(self) -> List<Int>:
        let m = links.len
        let top = max(max_len[v] for v in 0..m) ?? 0
        var cnt = List.filled(top + 1, 0)
        for v in 0..m: cnt[max_len[v]] += 1
        for i in 1..=top: cnt[i] += cnt[i - 1]
        var order = List.filled(m, 0)
        for v in (0..m).rev:
            let l = max_len[v]
            cnt[l] -= 1
            order[cnt[l]] = v
        order

    /// Number of end positions of each state's substrings in the indexed string. O(states).
    @allow(unproven_list_index)
    pub fn occurrences(self) -> List<Int>:
        var occ = [Int(p) for p in prefix]
        let order = self.topo_order()
        for i in (1..order.len).rev:
            let v = order[i]
            let p = links[v]
            if p != -1: occ[p] += occ[v]
        occ

    /// Longest common substring of the indexed string and `other`: returns `(len, end)` where
    /// `other[end - len..end]` is a longest common substring. O(|other|).
    @allow(unproven_list_index)
    pub fn lcs(self, other: List<A>) -> (len: Int, end: Int):
        var (v, l, best_len, best_end) = (0, 0, 0, 0)
        for i in 0..other.len:
            let c = other[i].to_idx
            while v != 0 and next[v * alpha + c] == -1:
                v = links[v]
                l = max_len[v]
            let nxt = next[v * alpha + c]
            if nxt != -1:
                v = nxt
                l += 1
            if l > best_len:
                best_len = l
                best_end = i + 1
        (len: best_len, end: best_end)

/// Online palindromic tree (eertree) over alphabet domain `A`, with `diff` and `series_link`
/// for O(n log n) palindromic partition DP.
/// State `0` is the odd root (`pal_len == -1`); state `1` is the even root (`pal_len == 0`).
pub struct PalindromicTree<A>:
    alpha: Int
    next: List<Int>
    slink: List<Int>
    plen: List<Int>
    diff_val: List<Int>
    ser_link: List<Int>
    cnt: List<Int>
    chars: List<A>
    last: Int

    /// Create an empty palindromic tree (eertree) over alphabet `A`. O(1).
    pub fn new() -> PalindromicTree<A>:
        let k = A.K
        PalindromicTree(k, List.filled(2 * k, -1), [0, 0], [-1, 0], [0, 0], [0, 0], [0, 0], [], 1)

    /// Build the palindromic tree of `s`. O(|s| * |A|) flat memory, O(|s|) amortized time.
    pub fn of(s: List<A>) -> PalindromicTree<A>:
        var pt = PalindromicTree<A>.new()
        for a in s: let _ = pt.push(a)
        pt

    /// Append one letter `a` and return the state of the longest palindromic suffix.
    /// O(1) amortized.
    @allow(unproven_list_index)
    pub fn push(mut self, a: A) -> Int:
        let c = a.to_idx
        let pos = chars.len
        chars.push(a)
        unchecked:
            var cur = last
            while true:
                let prev = pos - plen[cur] - 1
                if prev >= 0 and chars[prev] == a: break
                cur = slink[cur]
            if next[cur * alpha + c] == -1:
                let nw = slink.len
                let nw_len = plen[cur] + 2
                let link_to = if cur == 0:
                    1
                else:
                    var f = slink[cur]
                    while true:
                        let prev = pos - plen[f] - 1
                        if prev >= 0 and chars[prev] == a: break
                        f = slink[f]
                    next[f * alpha + c]
                let d = nw_len - plen[link_to]
                let sl = if d == diff_val[link_to]: ser_link[link_to] else: link_to
                for _ in 0..alpha: next.push(-1)
                slink.push(link_to)
                plen.push(nw_len)
                diff_val.push(d)
                ser_link.push(sl)
                cnt.push(0)
                next[cur * alpha + c] = nw
            last = next[cur * alpha + c]
            cnt[last] += 1
        last

    /// Total number of states, including the odd (`0`) and even (`1`) roots. O(1).
    pub @get fn len(self) -> Int = slink.len

    /// Number of distinct non-empty palindromic substrings. O(1).
    pub @get fn distinct(self) -> Int = slink.len - 2

    /// Length of the palindrome represented by `state` (`-1` for odd root, `0` for even root). O(1).
    @allow(unproven_list_index)
    pub fn pal_len(self, state: Int) -> Int = plen[state]

    /// Longest proper palindromic suffix state of `state`. O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> Int = slink[state]

    /// Transition from `state` wrapping with `a` on both ends, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> ?Int:
        let t = next[state * alpha + a.to_idx]
        if t == -1: none else: t

    /// Period `pal_len(state) - pal_len(link(state))`. O(1).
    @allow(unproven_list_index)
    pub fn diff(self, state: Int) -> Int = diff_val[state]

    /// Nearest suffix-link ancestor with a different `diff`, for O(log n) series DP. O(1).
    @allow(unproven_list_index)
    pub fn series_link(self, state: Int) -> Int = ser_link[state]

    /// Total occurrences of each palindrome state in the pushed string. O(states).
    @allow(unproven_list_index)
    pub fn occurrences(self) -> List<Int>:
        var occ = [x for x in cnt]
        unchecked:
            for v in (2..occ.len).rev: occ[slink[v]] += occ[v]
        occ

/// Dynamic trie over alphabet domain `A`. O(|word|) per operation; flat `|A|`-stride table.
pub struct Trie<A>:
    alpha: Int
    next: List<Int>
    ends: List<Int>
    pass: List<Int>

    /// Create an empty prefix trie over alphabet `A`. O(1).
    pub fn new() -> Trie<A>:
        let k = A.K
        Trie(k, List.filled(k, -1), [0], [0])

    /// Root node (empty word): always `0`. O(1).
    pub @get fn root(self) -> Int = 0

    /// Total number of nodes, root included. O(1).
    pub @get fn len(self) -> Int = ends.len

    /// Child of `node` along edge `a`, or `none`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, node: Int, a: A) -> ?Int:
        let t = next[node * alpha + a.to_idx]
        if t == -1: none else: t

    /// Node of `word`, or `none` if no added word starts with it. O(|word|).
    @allow(unproven_list_index)
    pub fn find(self, word: List<A>) -> ?Int:
        var at = 0
        for a in word:
            let t = next[at * alpha + a.to_idx]
            if t == -1: return none
            at = t
        at

    /// Insert one copy of `word` and return its terminal node. O(|word|).
    @allow(unproven_list_index)
    pub fn add(mut self, word: List<A>) -> Int:
        var at = 0
        pass[0] += 1
        for a in word:
            let idx = at * alpha + a.to_idx
            if next[idx] == -1:
                let nw = ends.len
                for _ in 0..alpha: next.push(-1)
                ends.push(0)
                pass.push(0)
                next[idx] = nw
            at = next[idx]
            pass[at] += 1
        ends[at] += 1
        at

    /// Remove one copy of `word`. Returns `false` (no change) if absent. O(|word|).
    @allow(unproven_list_index)
    pub fn remove(mut self, word: List<A>) -> Bool:
        let node = self.find(word)
        if node == none or ends[node!] == 0: return false
        var at = 0
        pass[0] -= 1
        for a in word:
            at = next[at * alpha + a.to_idx]
            pass[at] -= 1
        ends[at] -= 1
        true

    /// Copies of `word` currently in the trie. O(|word|).
    @allow(unproven_list_index)
    pub fn count(self, word: List<A>) -> Int:
        let node = self.find(word)
        if node == none: 0 else: ends[node!]

    /// Added words starting with `prefix`. O(|prefix|).
    @allow(unproven_list_index)
    pub fn count_prefix(self, prefix: List<A>) -> Int:
        let node = self.find(prefix)
        if node == none: 0 else: pass[node!]

    /// Words ending at `node`. O(1).
    @allow(unproven_list_index)
    pub fn ends_at(self, node: Int) -> Int = ends[node]

    /// Words passing through or ending at `node`. O(1).
    @allow(unproven_list_index)
    pub fn pass_at(self, node: Int) -> Int = pass[node]

/// Aho-Corasick multi-pattern automaton over alphabet domain `A`.
/// Built in O(sum |p| * |A|) flat memory and time; `step`, `link`, and `hit` are O(1).
pub struct AhoCorasick<A>:
    alpha: Int
    goto: List<Int>
    fail: List<Int>
    ends: List<List<Int>>
    exit: List<Int>
    order: List<Int>
    lens: List<Int>

    @allow(unproven_list_index)
    /// Build an Aho-Corasick automaton over `patterns`. O(total pattern length).
    pub fn new(patterns: List<List<A>>) -> AhoCorasick<A>:
        let k = A.K
        var goto = List.filled(k, -1)
        var ends: List<List<Int>> = [[]]
        var lens = List.filled(patterns.len, 0)
        for i in 0..patterns.len:
            lens[i] = patterns[i].len
            var at = 0
            for a in patterns[i]:
                let idx = at * k + a.to_idx
                if goto[idx] == -1:
                    let nw = ends.len
                    for _ in 0..k: goto.push(-1)
                    ends.push([])
                    goto[idx] = nw
                at = goto[idx]
            ends[at].push(i)
        let n = ends.len
        var fail = List.filled(n, 0)
        var exit = List.filled(n, -1)
        var order = List.filled(n, 0)
        var head = 0
        var tail = 1
        while head < tail:
            let s = order[head]
            head += 1
            let base = s * k
            let fbase = fail[s] * k
            for c in 0..k:
                let past = if s == 0: 0 else: goto[fbase + c]
                let t = goto[base + c]
                if t != -1:
                    fail[t] = past
                    exit[t] = if ends[past].is_empty: exit[past] else: past
                    order[tail] = t
                    tail += 1
                else:
                    goto[base + c] = past
        AhoCorasick(k, goto, fail, ends, exit, order, lens)

    /// Root state (before reading any letter): always `0`. O(1).
    pub @get fn root(self) -> Int = 0

    /// Number of states. O(1).
    pub @get fn len(self) -> Int = fail.len

    /// Total transition from `state` on `a`. O(1).
    @allow(unproven_list_index)
    pub fn step(self, state: Int, a: A) -> Int = goto[state * alpha + a.to_idx]

    /// Failure (suffix) link of `state` (root links to `0`). O(1).
    @allow(unproven_list_index)
    pub fn link(self, state: Int) -> Int = fail[state]

    /// True when any pattern ends at `state` or along its exit links. O(1).
    @allow(unproven_list_index)
    pub fn hit(self, state: Int) -> Bool = not ends[state].is_empty or exit[state] != -1

    /// All pattern indices ending at `state`, longest first. O(output).
    @allow(unproven_list_index)
    pub fn words_at(self, state: Int) -> List<Int>:
        var found: List<Int> = []
        var at = state
        while at != -1:
            for w in ends[at]: found.push(w)
            at = exit[at]
        found

    /// Yield every occurrence `(at, pattern)` in `text`, ordered by end position.
    /// O(|text| + output).
    @allow(unproven_list_index)
    pub fn matches(self, text: List<A>) -> yields (at: Int, pattern: Int):
        var state = 0
        for i in 0..text.len:
            state = goto[state * alpha + text[i].to_idx]
            var at = state
            while at != -1:
                for w in ends[at]: yield (at: i + 1 - lens[w], pattern: w)
                at = exit[at]

    /// Occurrence count of every pattern in `text` (overlaps included). O(|text| + states).
    @allow(unproven_list_index)
    pub fn counts(self, text: List<A>) -> List<Int>:
        var visits = List.filled(fail.len, 0)
        var state = 0
        for a in text:
            state = goto[state * alpha + a.to_idx]
            visits[state] += 1
        for i in (1..order.len).rev: visits[fail[order[i]]] += visits[order[i]]
        var total = List.filled(lens.len, 0)
        for s in 0..fail.len:
            for w in ends[s]: total[w] = visits[s]
        total

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
        want.sort(by: i => [s[j] for j in i..s.len])
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

@test
fn borders_prefix_matches_and_overlap_match_brute():
    assert [k for k in borders("abacaba")] == [3, 1]
    assert [k for k in borders("aaaa")] == [3, 2, 1]
    assert [k for k in borders("")] == []
    assert "ababab".prefix_matches("aba") == [1, 2, 3, 2, 3, 2]
    assert overlap("abcab", "cabx") == 3
    assert overlap("", "abc") == 0 and overlap("abc", "") == 0
    for _ in 0..40:
        let s = random_word(rand(0, 12), 2)
        let n = s.len
        let want_b = [k for k in (1..n).rev if s[0..k] == s[n - k..n]]
        assert [k for k in borders(s)] == want_b
        let pat = random_word(rand(1, 5), 2)
        let pm = s.prefix_matches(pat)
        for i in 0..n:
            var want = 0
            for k in 1..=min(pat.len, i + 1):
                if s[i + 1 - k..i + 1] == pat[0..k]: want = k
            assert pm[i] == want
        var want_ov = 0
        for k in 1..=min(n, pat.len):
            if s[n - k..n] == pat[0..k]: want_ov = k
        assert overlap(s, pat) == want_ov

@test
fn lyndon_factorization_and_min_rotation_match_definitions():
    assert lyndon_factorization("") == []
    assert lyndon_factorization("abab") == [0, 2]
    assert lyndon_factorization("cba") == [0, 1, 2]
    assert min_rotation("") == 0
    assert min_rotation("cba") == 2
    assert min_rotation("baca") == 3
    assert min_rotation("aaaa") == 0
    for _ in 0..40:
        let s = random_word(rand(1, 14), 3)
        let n = s.len
        let starts = lyndon_factorization(s)
        assert starts[0] == 0
        var prev: ?List<Char> = none
        for idx in 0..starts.len:
            let lo = starts[idx]
            let hi = if idx + 1 < starts.len: starts[idx + 1] else: n
            let w = [s[j] for j in lo..hi]
            // Each factor is Lyndon: strictly less than all its non-empty proper suffixes
            for cut in 1..w.len:
                assert w < [w[j] for j in cut..w.len]
            if let p = prev: assert p >= w
            prev = w
        // min_rotation matches brute force over all cyclic shifts
        var best = 0
        var best_rot = [s[j] for j in 0..n]
        for r in 1..n:
            let rot = [s[(r + j) % n] for j in 0..n]
            if rot < best_rot:
                best = r
                best_rot = rot
        assert min_rotation(s) == best

@test
fn manacher_and_is_palindrome_match_brute():
    assert is_palindrome("") and is_palindrome("a") and is_palindrome("abba")
    assert not is_palindrome("ab") and not is_palindrome("abca")
    let mp = manacher("abacaba")
    assert mp.odd == [1, 2, 1, 4, 1, 2, 1]
    assert mp.longest == (l: 0, r: 7)
    assert manacher("abba").even == [0, 0, 2, 0]
    assert manacher("").count == 0 and manacher("").longest == (l: 0, r: 0)
    for _ in 0..40:
        let s = random_word(rand(0, 14), 2)
        let n = s.len
        let p = manacher(s)
        var total = 0
        var best = (l: 0, r: 0)
        for l in 0..n:
            for r in l..=n:
                let ok = is_palindrome([s[j] for j in l..r])
                assert p.is_palindrome(l..r) == ok
                if r > l and ok:
                    total += 1
                    if r - l > best.r - best.l: best = (l: l, r: r)
        assert p.count == total
        assert p.longest == best

@test
fn suffix_array_struct_queries_match_brute():
    let empty = SuffixArray<Char>("")
    assert empty.len == 0 and empty.distinct == 0 and empty.count("a") == 0
    for _ in 0..30:
        let s = random_word(rand(1, 16), 3)
        let n = s.len
        let sa = SuffixArray(s)
        assert sa.len == n
        for i in 0..n:
            for j in 0..n:
                assert sa.lcp_between(i, j) == common_prefix(s, i, j)
        // matches and count
        for _ in 0..10:
            let pat = random_word(rand(0, 4), 3)
            let r = sa.matches(pat)
            let want = if pat.is_empty: [i for i in 0..n] else: [
                i for i in 0..=n - pat.len if s[i..i + pat.len] == pat
            ]
            assert sa.count(pat) == want.len
            var got = [sa.sa[idx] for idx in r]
            got.sort()
            assert got == want
        // lcp_range
        for _ in 0..10:
            let i = rand(0, n - 1)
            let len = rand(0, n - i + 1)
            let r = sa.lcp_range(i, len)
            var want: List<Int> = [sa.rank[j] for j in 0..n if common_prefix(s, i, j) >= len]
            want.sort()
            assert [idx for idx in r] == want
        // compare
        for _ in 0..20:
            let l1 = rand(0, n)
            let r1 = rand(l1, n)
            let l2 = rand(0, n)
            let r2 = rand(l2, n)
            let a = [s[j] for j in l1..r1]
            let b = [s[j] for j in l2..r2]
            let want = if a < b: -1 elif a > b: 1 else: 0
            assert sa.compare(l1, r1, l2, r2) == want
        // distinct substrings
        var seen = Set<List<Char>>()
        for l in 0..n:
            for r in l + 1..=n: seen.add([s[j] for j in l..r])
        assert sa.distinct == seen.len

@test
fn rolling_hash_lcp_lcs_and_hash2d_match_brute():
    for _ in 0..25:
        let s = random_word(rand(1, 14), 2)
        let t = random_word(rand(1, 14), 2)
        let hs = RollingHash(s)
        let ht = RollingHash.with_bases(t, hs.bases)
        for i in 0..=s.len:
            for j in 0..=s.len:
                assert hs.lcp(i, j) == common_prefix(s, i, j)
                var cs = 0
                while cs < i and cs < j and s[i - 1 - cs] == s[j - 1 - cs]: cs += 1
                assert hs.lcs(i, j) == cs
        for _ in 0..20:
            let a = rand(0, s.len)
            let b = rand(0, t.len)
            var k = 0
            while a + k < s.len and b + k < t.len and s[a + k] == t[b + k]: k += 1
            assert hs.lcp_with(a, ht, b) == k
    // Hash2D matches brute rectangle equality
    domain R = 0..5
    domain C = 0..6
    let g = [R, C] (r, c) => rand(0, 1)
    let h2 = Hash2D<R, C>((r, c) => g[r, c])
    for _ in 0..100:
        let (hgt, wid) = (rand(1, 3), rand(1, 3))
        let (r1, c1, r2, c2) = (
            rand(0, 5 - hgt),
            rand(0, 6 - wid),
            rand(0, 5 - hgt),
            rand(0, 6 - wid),
        )
        let same = all(
            g[R(r1 + i), C(c1 + j)] == g[R(r2 + i), C(c2 + j)] for i in 0..hgt for j in 0..wid
        )
        let a = h2.rect(R(r1)..=R(r1 + hgt - 1), C(c1)..=C(c1 + wid - 1))
        let b = h2.rect(R(r2)..=R(r2 + hgt - 1), C(c2)..=C(c2 + wid - 1))
        assert (a == b) == same

@test
fn suffix_automaton_matches_brute_and_linear_occurrences():
    domain A = 0..2
    for _ in 0..30:
        let n = rand(0, 10)
        let s = [A(rand(0, 1)) for _ in 0..n]
        let sam = SuffixAutomaton<A>.of(s)
        let occ = sam.occurrences()
        let order = sam.topo_order()
        assert order.len == sam.len
        for i in 1..order.len: assert sam.longest(order[i - 1]) <= sam.longest(order[i])
        for v in 1..sam.len:
            assert sam.shortest(v) == sam.longest(sam.link(v)!) + 1
            assert sam.shortest(v) <= sam.longest(v)
        for len in 1..=4:
            for code in 0..(1 << len):
                let w = [A((code >> i) % 2) for i in 0..len]
                let times = sum(Int(all(s[i + j] == w[j] for j in 0..len)) for i in 0..=n - len)
                assert sam.contains(w) == (times > 0)
                if let v = sam.find(w):
                    assert occ[v] == times
                    if len == sam.longest(v):
                        var first = -1
                        for i in 0..=n - len:
                            if all(s[i + j] == w[j] for j in 0..len):
                                first = i + len - 1
                                break
                        assert sam.first_pos(v) == first
        var seen = Set<List<A>>()
        for l in 0..n:
            for r in l + 1..=n: seen.add([s[j] for j in l..r])
        assert sam.distinct == seen.len
        assert sam.len <= max(2 * n, 1)
        let m = rand(0, 10)
        let t = [A(rand(0, 1)) for _ in 0..m]
        let (lcs_len, lcs_end) = sam.lcs(t)
        var brute_lcs = 0
        for i in 0..m:
            for j in i + 1..=m:
                if [t[k] for k in i..j] in seen: brute_lcs max= j - i
        assert lcs_len == brute_lcs
        if lcs_len > 0: assert [t[k] for k in lcs_end - lcs_len..lcs_end] in seen

@test
fn palindromic_tree_distinct_occurrences_and_series_dp():
    domain A = 0..2
    for _ in 0..30:
        let n = rand(0, 12)
        let s = [A(rand(0, 1)) for _ in 0..n]
        var pt = PalindromicTree<A>.new()
        let inf = 1_000_000
        var dp = List.filled(n + 1, inf)
        var series_dp = List.filled(n + 3, inf)
        dp[0] = 0
        for i in 0..n:
            var v = pt.push(s[i])
            while pt.pal_len(v) > 0:
                series_dp[v] = dp[i + 1 - pt.pal_len(pt.series_link(v)) - pt.diff(v)]
                if pt.diff(v) == pt.diff(pt.link(v)):
                    series_dp[v] min= series_dp[pt.link(v)]
                dp[i + 1] min= series_dp[v] + 1
                v = pt.series_link(v)
        // Brute distinct palindromes and total palindromic substrings
        var pals = Set<List<A>>()
        var total_pals = 0
        for l in 0..n:
            for r in l + 1..=n:
                let sub = [s[j] for j in l..r]
                if is_palindrome(sub):
                    pals.add(sub)
                    total_pals += 1
        assert pt.distinct == pals.len
        let occ = pt.occurrences()
        assert sum(occ[v] for v in 2..pt.len) == total_pals
        // Brute minimum palindromic partition DP
        var brute = List.filled(n + 1, inf)
        brute[0] = 0
        for r in 1..=n:
            for l in 0..r:
                if is_palindrome([s[j] for j in l..r]): brute[r] min= brute[l] + 1
        assert dp[n] == brute[n]

@test
fn trie_add_remove_and_prefix_counts():
    domain A = 0..3
    for _ in 0..25:
        var t = Trie<A>()
        var words: List<List<A>> = []
        for _ in 0..30:
            let w = [A(rand(0, 2)) for _ in 0..rand(0, 4)]
            if rand(0, 3) > 0:
                let node = t.add(w)
                assert t.ends_at(node) >= 1 and t.pass_at(node) >= 1
                words.push(w)
            else:
                let i = words.find(w)
                assert t.remove(w) == (i != none)
                if i != none: words = [words[j] for j in 0..words.len if j != i!]
            let q = [A(rand(0, 2)) for _ in 0..rand(0, 3)]
            assert t.count(q) == sum(Int(w == q) for w in words)
            assert t.count_prefix(q) == sum(
                Int(w.len >= q.len and all(w[i] == q[i] for i in 0..q.len)) for w in words
            )

@test
fn aho_corasick_matches_counts_and_avoidance_dp():
    domain A = 0..2
    for _ in 0..30:
        let patterns = [[A(rand(0, 1)) for _ in 0..rand(1, 4)] for _ in 0..rand(1, 5)]
        let text = [A(rand(0, 1)) for _ in 0..rand(0, 14)]
        let ac = AhoCorasick<A>(patterns)
        var want: List<(Int, Int)> = []
        for i in 0..text.len:
            for w in 0..patterns.len:
                let k = patterns[w].len
                if i + k <= text.len and all(
                    text[i + j] == patterns[w][j] for j in 0..k
                ): want.push((i, w))
        let got = [(m.at, m.pattern) for m in ac.matches(text)]
        assert got.sorted() == want.sorted()
        assert ac.counts(text) == [sum(Int(p.1 == w) for p in want) for w in 0..patterns.len]
        assert ac.len <= 1 + sum(w.len for w in patterns)
    // Avoidance DP test
    let pats = [[A(0), A(1)], [A(1), A(1)]]
    let ac = AhoCorasick<A>(pats)
    var ways = List.filled(ac.len, 0)
    ways[ac.root] = 1
    for _ in 0..6:
        var nxt = List.filled(ac.len, 0)
        for s in 0..ac.len:
            for a in A:
                let t = ac.step(s, a)
                if not ac.hit(t): nxt[t] += ways[s]
        ways = nxt
    var brute = 0
    for code in 0..(1 << 6):
        let text = [A((code >> i) % 2) for i in 0..6]
        if [m for m in ac.matches(text)].is_empty: brute += 1
    assert ways.sum() == brute

@test
fn indexed_views_and_common_prefix_suffix():
    assert common_prefix_len("abacaba", "abax") == 3
    assert "abacaba".common_prefix_len("abax") == 3
    assert common_suffix_len("abacaba", "zzcaba") == 4
    assert "abacaba".common_suffix_len("zzcaba") == 4
    assert common_prefix_len("abacaba"[2..7], "acaz") == 3
    assert common_suffix_len("abacaba"[0..5], "xxaca") == 3
    assert common_prefix_len([1, 2, 3], [1, 2, 4, 5]) == 2
    assert common_suffix_len([1, 2, 3], [9, 2, 3]) == 2
    assert period("xxabababyy"[2..8]) == 2
    assert [i for i in "ababa".find_all("aba")] == [0, 2]
    assert [i for i in "xxababaxx"[2..7].find_all("aba")] == [0, 2]
    assert "ababa".prefix_matches("aba") == [1, 2, 3, 2, 3]
    assert overlap("abcab", "abxyz") == 2
    assert lyndon_factorization("bbab") == [0, 1, 2]
    assert min_rotation("cab") == 1
    let pal = manacher("abacaba")
    assert pal.is_palindrome(0..7)
    assert pal.is_palindrome(1..6)
    assert pal.is_palindrome(1..=5)
    assert not pal.is_palindrome(0..4)
    domain Pos = 1..=5
    let arr: [Pos]Char = [Pos] i => "ababa"[i.ord]
    assert manacher(arr).is_palindrome(Pos(1)..=Pos(5))
    assert not manacher(arr).is_palindrome(Pos(1)..=Pos(4))
    assert period(arr) == 2

@test
fn lyndon_runs_matches_brute():
    fn brute_runs(s: List<Int>) -> List<(l: Int, r: Int, period: Int)>:
        let n = s.len
        var out = List<(l: Int, r: Int, period: Int)>()
        for l in 0..n:
            for r in l + 2..=n:
                var min_p = 0
                for p in 1..=(r - l) / 2:
                    if all(s[i] == s[i + p] for i in l..r - p):
                        min_p = p
                        break
                if min_p == 0: continue
                if l > 0 and s[l - 1] == s[l - 1 + min_p]: continue
                if r < n and s[r] == s[r - min_p]: continue
                out.push((l: l, r: r, period: min_p))
        out

    assert lyndon_runs("").is_empty
    assert lyndon_runs("a").is_empty
    assert lyndon_runs("aa") == [(l: 0, r: 2, period: 1)]
    assert lyndon_runs("ababa") == [(l: 0, r: 5, period: 2)]
    var seed = 424242
    for n in 1..=14:
        for _ in 0..6:
            var s = List<Int>()
            for _ in 0..n:
                seed = (seed * 1103515245 + 12345) & 0x7fffffff
                s.push((seed >> 16) % 3)
            assert lyndon_runs(s) == brute_runs(s)

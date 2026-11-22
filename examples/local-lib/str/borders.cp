//! Borders of a string (prefixes that are also suffixes), its periods, and how far the
//! end of one string overlaps the start of another.
use std.strings.{prefix_function}

/// The lengths of every border of `s`, longest first, excluding `s` itself and the empty
/// border. O(n).
pub fn borders<T>(s: List<T>) -> List<Int>:
    if s.is_empty: return []
    let pi = prefix_function(s)
    var out: List<Int> = []
    var k = pi[s.len - 1]
    while k > 0:
        out.push(k)
        k = pi[k - 1]
    out

/// The shortest period of `s`: the least `p >= 1` with `s[i] == s[i + p]` wherever both exist.
pub fn period<T>(s: List<T>) -> Int:
    if s.is_empty: return 0
    s.len - prefix_function(s)[s.len - 1]

/// The length of the longest suffix of `a` that is a prefix of `b` (at most the shorter
/// length). O(|a| + |b|). `a + b[overlaps(a, b)..]` is the shortest string that ends
/// with `b` and starts with `a`.
pub fn overlaps<T>(a: List<T>, b: List<T>) -> Int:
    let keep = min(a.len, b.len)
    // b's prefix, then a's tail: positions are compared, so no separator value is needed
    var k = 0
    let pi = prefix_function([b[i] for i in 0..keep])
    for i in a.len - keep..a.len:
        while k > 0 and (k == keep or a[i] != b[k]): k = pi[k - 1]
        if a[i] == b[k]: k += 1
    k

@test
fn borders_and_periods_match_their_definitions():
    for _ in 0..80:
        let s = [rand(0, 1) for _ in 0..rand(0, 10)]
        let n = s.len
        let want = [k for k in (1..n).rev if (0..k).all(i => s[i] == s[n - k + i])]
        assert borders(s) == want
        if n > 0:
            assert period(s) == (1..=n).find(p => (0..n - p).all(i => s[i] == s[i + p]))

@test
fn overlap_is_the_longest_suffix_prefix():
    for _ in 0..120:
        let a = [rand(0, 1) for _ in 0..rand(0, 8)]
        let b = [rand(0, 1) for _ in 0..rand(0, 8)]
        var want = 0
        for k in 1..=min(a.len, b.len):
            if (0..k).all(i => a[a.len - k + i] == b[i]): want = k
        assert overlaps(a, b) == want
    assert overlaps([c for c in "abcab"], [c for c in "cabx"]) == 3

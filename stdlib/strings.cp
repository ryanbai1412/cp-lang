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

/// Palindrome radii (Manacher). `odd[i]` is the number of palindromes centred on `s[i]`;
/// `even[i]` the number centred between `s[i - 1]` and `s[i]`. O(n).
pub fn palindromes<T>(s: List<T>) -> (odd: List<Int>, even: List<Int>):
    let n = s.len
    var (odd, even) = (List.filled(n, 0), List.filled(n, 0))
    var (l, r) = (0, -1)
    for i in 0..n:
        var k = if i > r: 1 else: min(odd[l + r - i], r - i + 1)
        while i - k >= 0 and i + k < n and s[i - k] == s[i + k]: k += 1
        odd[i] = k
        if i + k - 1 > r: (l, r) = (i - k + 1, i + k - 1)
    (l, r) = (0, -1)
    for i in 0..n:
        var k = if i > r: 0 else: min(even[l + r - i + 1], r - i + 1)
        while i - k - 1 >= 0 and i + k < n and s[i - k - 1] == s[i + k]: k += 1
        even[i] = k
        if i + k - 1 > r: (l, r) = (i - k, i + k - 1)
    (odd: odd, even: even)

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
fn palindromes_match_expanding_from_each_centre():
    for _ in 0..40:
        let s = random_word(rand(0, 16), 2)
        let n = s.len
        let p = palindromes(s)
        for i in 0..n:
            var k = 0
            while i - k >= 0 and i + k < n and s[i - k] == s[i + k]: k += 1
            assert p.odd[i] == k
            k = 0
            while i - k - 1 >= 0 and i + k < n and s[i - k - 1] == s[i + k]: k += 1
            assert p.even[i] == k

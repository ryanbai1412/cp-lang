//! Linear-time palindrome radii (Manacher's algorithm).

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

@test
fn palindromes_match_expanding_from_each_centre():
    assert palindromes("abacaba").odd == [1, 2, 1, 4, 1, 2, 1]
    assert palindromes("abba").even == [0, 0, 2, 0]
    for _ in 0..40:
        let s = [Char(Int('a') + rand(0, 1)) for _ in 0..rand(0, 16)]
        let n = s.len
        let p = palindromes(s)
        for i in 0..n:
            var k = 0
            while i - k >= 0 and i + k < n and s[i - k] == s[i + k]: k += 1
            assert p.odd[i] == k
            k = 0
            while i - k - 1 >= 0 and i + k < n and s[i - k - 1] == s[i + k]: k += 1
            assert p.even[i] == k

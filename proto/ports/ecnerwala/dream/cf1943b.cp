// Codeforces 1943 B: Non-Palindromic Substring.
//
// t = s[l..=r] of length m is k-good when some length-k window of t is not a
// palindrome; the answer is the sum of the good k. Length 1 is never good and
// length m is good iff t is not a palindrome. For 2 <= k < m, every length-k
// window is a palindrome iff t has period 2 (k odd) or period 1 (k even):
// two overlapping palindromic windows reflect t[i] onto t[i + 2] (resp.
// t[i + 1]). Each query is two period tests, one palindrome test and two
// arithmetic series, all O(1).
@!multitest
use std.strings.{manacher}
use std.prefix.{prefix1}

let (n, q): (2..=2e5, 1..=2e5) = read()
domain Pos = 1..=n
let s: [Pos]Char = read()
let pal = manacher(s)

// t has period d iff no position past its first d differs from the one d before it.
let differs = [1..=2] d => prefix1([Pos] i => Int(s[i] != s[i.back(d) ?? i]))
fn has_period(t: Range<Pos>, d: 1..=2) -> Bool = differs[d].sum(t.drop(d)) == 0

repeat q:
    let (l, r): (Pos, Pos) = read()
    let t = l..=r
    let m = t.len
    let even = if has_period(t, 1): 0 else: (2..m).step(2).sum()
    let odd = if has_period(t, 2): 0 else: (3..m).step(2).sum()
    print(even + odd + (if pal.is_palindrome(t): 0 else: m))

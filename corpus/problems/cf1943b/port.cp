// Source: Codeforces 1943B (Non-Palindromic Substring),
//   https://codeforces.com/contest/1943/problem/B
// limits: 3000 ms, 250 MB
// Problem: t is k-good if some length-k substring of t is not a palindrome; f(t) sums the good k.
//   For a string s (n <= 2e5) answer q <= 2e5 queries f(s[l..=r]), l < r. Multitest, sums <= 2e5.
// Solution: k = 1 is never good and k = m = |t| is good iff t is not a palindrome. For 2 <= k < m,
//   all length-k windows are palindromes iff t has period 2 (k odd) or period 1 (k even): two
//   overlapping palindromic windows reflect t[i] onto t[i + 2] (resp. t[i + 1]). Prefix counts of
//   s[i] != s[i - d] test the periods, Manacher the palindrome, and the good k form two arithmetic
//   series: O(n) setup, O(1) per query.
@!multitest
use std.strings.{manacher}

let (n, q): (2..=2e5, 1..=2e5) = read()
domain Pos = 1..=n
let s: [Pos]Char = read()
let pal = manacher(s)

// t has period d iff no position past its first d differs from the one d before it.
let differs = [1..=2] d => prefix([Pos] i => Int(i.back(d).map(p => s[p] != s[i]) ?? false))
fn has_period(t: Range<Pos>, d: 1..=2) -> Bool = differs[d].fold(t.drop(d)) == 0

repeat q:
    let (l, r): (Pos, Pos) = read()
    let t = l..=r
    let m = t.len
    let even = if has_period(t, 1): 0 else: (2..m).step(2).sum()
    let odd = if has_period(t, 2): 0 else: (3..m).step(2).sum()
    print(even + odd + (if pal.is_palindrome(t): 0 else: m))

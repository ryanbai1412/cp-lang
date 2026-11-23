// Source: NOIP 2020 P2 (String Matching), https://dmoj.ca/problem/noip20p2
// Problem: for a lowercase string S (|S| <= 2^20), count the ways to write S = (AB)^i C with
//   A, B, C non-empty and i >= 1, where F(A) <= F(C) and F(X) is the number of letters occurring
//   an odd number of times in X. Multitest (T <= 5).
// Solution: fix w = |AB|. The Z-function gives how many times the prefix of length w repeats;
//   C must stay non-empty. (AB)^2 has all counts even, so F(C) is F of the suffix after w for
//   odd i and after 2w for even i. at_most[d] counts prefixes A of length < w with F(A) <= d,
//   grown by one prefix per w. O(26 n).
// original: corpus/submissions/5445673.cpp
use std.strings.{z_function}

domain Letter = 'a'..='z'

@multitest
let s: Str = read()
let n = s.len
let z = z_function(s)

var odd_from: [0..=n]Int = 0
var odd_to: [0..=n]Int = 0
var odd = Marks<Letter>()
for i in (0..n).rev:
    let c = Letter(s[i])
    if c in odd: odd.remove(c) else: odd.add(c)
    odd_from[i] = odd.len
odd.clear()
for i in 0..n:
    let c = Letter(s[i])
    if c in odd: odd.remove(c) else: odd.add(c)
    odd_to[i + 1] = odd.len

var at_most: [0..=26]Int = 0
var ways = 0
for w in 2..n:
    for d in odd_to[w - 1]..=26: at_most[d] += 1
    let copies = min(1 + z[w] / w, (n - 1) / w)
    let (odd_k, even_k) = ((copies + 1) / 2, copies / 2)
    ways += odd_k * at_most[odd_from[w]]
    if even_k > 0: ways += even_k * at_most[odd_from[2 * w]]
print(ways)

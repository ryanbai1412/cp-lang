// Source: Mock CCC '21 S2 (Colorful Strings), https://dmoj.ca/problem/nccc8s2
// limits: 250 ms, 1024 MB
// Problem: a lowercase string of length <= 1e5. Count, with multiplicity, its subsequences in
//   which every letter appears at most once (the empty one included), mod 1e9 + 7.
// Solution: each letter is either left out or taken from one of its cnt occurrences, so the
//   answer is the product of (cnt[c] + 1) over the 26 letters. O(|s|).
// original: corpus/submissions/3381135.java
type Z = Mod<1_000_000_007>
let s: [domain P]Char = read()
assert 1 <= P.count <= 100_000
domain Letter = 'a'..='z'
var count: [Letter]Int = 0
for c in s: count[Letter(c)] += 1
print(prod(Z(count[c] + 1) for c in Letter))

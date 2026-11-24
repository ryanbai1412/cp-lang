// Source: Mock CCC '22 Contest 1 J3 (String Crossing Maximization),
//   https://dmoj.ca/problem/mccc3j3
// Problem: uppercase strings S and T, |S|, |T| <= 1e6. A crossing is a pair (i, j) with
//   S_i = T_j. Change at most one character of T; print the most crossings.
// Solution: crossings are sum over letters c of cntS[c] * cntT[c]. Changing a T letter x into y
//   gains cntS[y] - cntS[x], so the best gain is max over all letters of cntS minus min over
//   letters in T of cntS, if positive. O(|S| + |T|).
// Bounds: n, m: 1..=1e6
// original: corpus/submissions/4223424.cpp
let (n, m): (1..=1_000_000, 1..=1_000_000) = read()
let s: [domain S]Char = read()
let t: [domain T]Char = read()
assert S.count == n and T.count == m
domain Letter = 'A'..='Z'
var in_s: [Letter]Int = 0
var in_t: [Letter]Int = 0
for c in s: in_s[Letter(c)] += 1
for c in t: in_t[Letter(c)] += 1

// turning one letter of t from x into y gains in_s[y] - in_s[x]
let gain = max(in_s[c] for c in Letter) - min(in_s[c] for c in Letter if in_t[c] > 0)
print(sum(in_s[c] * in_t[c] for c in Letter) + max(gain, 0))

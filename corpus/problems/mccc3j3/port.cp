// Source: Mock CCC '22 Contest 1 J3 (String Crossing Maximization),
//   https://dmoj.ca/problem/mccc3j3
// Problem: uppercase strings S and T, |S|, |T| <= 1e6. A crossing is a pair (i, j) with
//   S_i = T_j. Change at most one character of T; print the most crossings.
// Solution: crossings are sum over letters c of cntS[c] * cntT[c]. Changing a T letter x into y
//   gains cntS[y] - cntS[x], so the best gain is max over all letters of cntS minus min over
//   letters in T of cntS, if positive. O(|S| + |T|).
// original: corpus/submissions/4223424.cpp

let (n, m): (Int, Int) = read()
let a: Str = read()
let b: Str = read()

domain Letter = 'A'..='Z'
var acnt: [Letter]Int = [Letter] _ => 0
var bcnt: [Letter]Int = [Letter] _ => 0
for i in 0..n: acnt[Letter(a[i])] += 1
for i in 0..m: bcnt[Letter(b[i])] += 1

var base = 0
var hi = 0
var lo: ?Int = None
for c in Letter:
    base += acnt[c] * bcnt[c]
    hi max= acnt[c]
    if bcnt[c] > 0:
        if let cur = lo:
            if acnt[c] < cur: lo = acnt[c]
        else:
            lo = acnt[c]

var gain = 0
if let x = lo:
    if hi - x > 0: gain = hi - x
print(base + gain)

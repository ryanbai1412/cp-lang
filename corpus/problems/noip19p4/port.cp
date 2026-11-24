// Source: CSP-S 2019 Day 2 P1 (Emiya's meal), https://dmoj.ca/problem/noip19p4
// limits: 1000 ms, 250 MB
// Problem: a[i][j] < 998244353 distinct dishes use method i (n <= 100) and ingredient j
//   (m <= 2000). Count non-empty sets of dishes with pairwise different methods in which every
//   ingredient is used by at most half of the dishes, modulo 998244353.
// Solution: count all sets (prod over methods of (row sum + 1), minus 1) and subtract the bad
//   ones. Only one ingredient can exceed half, so fix it and DP over methods on its lead: dishes
//   with it minus dishes without it (each method adds one of either, or nothing). Bad sets end
//   with lead > 0. O(m n^2).
// Bounds: n: 1..=100; m: 1..=2000
// original: corpus/submissions/5147491.cpp
type Z = Mod<998244353>

let (n, m): (1..=100, 1..=2000) = read()
domain Method = 1..=n
domain Food = 1..=m
let a: [Method, Food]Z = read()
let row = [Method] i => sum(a[i, j] for j in Food)

domain Lead = -n..=n  // dishes using the fixed ingredient minus the other dishes
var bad = Z(0)
for j in Food:
    var ways = [Lead] d => Z(if d.label == 0: 1 else: 0)
    for i in Method:
        var next = ways.clone()
        for d in Lead:
            if let up = d.next: next[up] += ways[d] * a[i, j]
            if let down = d.prev: next[down] += ways[d] * (row[i] - a[i, j])
        ways = next
    bad += sum(ways[d] for d in Lead if d.label > 0)
print(([Method] i => row[i] + 1).prod() - 1 - bad)

// Source: CSP-S 2019 Day 2 P1 (Emiya's meal), https://dmoj.ca/problem/noip19p4
// Problem: a[i][j] < 998244353 distinct dishes use method i (n <= 100) and ingredient j
//   (m <= 2000). Count non-empty sets of dishes with pairwise different methods in which every
//   ingredient is used by at most half of the dishes, modulo 998244353.
// Solution: count all sets (prod over methods of (row sum + 1), minus 1) and subtract the bad
//   ones. Only one ingredient can exceed half, so fix it and DP over methods on its lead: dishes
//   with it minus dishes without it (each method adds one of either, or nothing). Bad sets end
//   with lead > 0. O(m n^2).
// original: corpus/submissions/5147491.cpp
type Z = Mod<998244353>

let (n, m): (Int, Int) = read()
domain Method = 0..n
domain Food = 0..m
let a: [Method, Food]Int = read()
let total = [Method] i => sum(a[i, j] for j in Food) % 998244353

domain Lead = -n..=n
var bad: Z = 0
for j in Food:
    var ways: [Lead]Z = [Lead] d => Z.zero
    ways[Lead(0)] = 1
    for i in Method:
        var next = [Lead] d => ways[d]
        for d in Lead:
            if let up = d.next: next[up] += ways[d] * Z(a[i, j])
            if let down = d.prev: next[down] += ways[d] * Z(total[i] - a[i, j])
        ways = next
    for d in Lead:
        if d.label > 0: bad += ways[d]

var all: Z = 1
for i in Method: all *= Z(total[i] + 1)
print(all - Z.one - bad)

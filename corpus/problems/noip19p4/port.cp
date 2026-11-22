// NOIP 2019 Day 2 P1, Emiya's meal: a[i][j] dishes use method i and
// ingredient j. Pick a non-empty set of dishes, at most one per method,
// such that no ingredient is in more than half of them; count the sets
// modulo 998244353.
//
// Count every non-empty choice, then subtract the bad ones. Only one
// ingredient can be in more than half, so fix it and track, method by
// method, its lead: dishes with it minus dishes without it.
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

// Source: CEOI 2018 Toys, https://dmoj.ca/problem/ceoi18p5
// limits: 2500 ms, 256 MB
// Problem: a collection with c_i identical toys of each type i has n = prod (c_i + 1) different
//   sub-collections (the empty one included). Given n <= 1e9, print how many totals sum c_i are
//   possible, then all of them in increasing order.
// Solution: a collection is a factorisation of n into factors >= 2, scoring sum (factor - 1). List
//   the divisors in O(sqrt n); split(rest, from, score) records rest as the last factor, then
//   tries each divisor d >= divs[from] with d^2 <= rest that divides it, so factors are
//   non-decreasing and each factorisation comes once. An ordered Set collects the scores; the cost
//   is the number of factorisations times the divisor scan.
// original: corpus/submissions/5323876.cpp
// Bounds: n: 1..=1e9
let n: 1..=1e9 = read()
let divs = divisors(n)
var found = Set<Int>()
// `rest` is still to be split into factors >= divs[from]; `score` so far.
fn split(rest: Int, from: Int, score: Int):
    found.insert(score + rest - 1)
    for k in from..divs.len:
        let d = divs[k]
        if d * d > rest: break
        if rest % d == 0: split(rest / d, k, score + d - 1)
split(n, 1, 0)
print(found.len)
print(found)

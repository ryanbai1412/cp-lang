// Source: IOI 2017 P4 (The Big Prize), https://dmoj.ca/problem/ioi17p4
// Problem: interactive. n <= 2e5 boxes hold prizes of types 1..v, one diamond (type 1),
//   and more than k^2 of type t when type t - 1 has k. ask(i) returns how many boxes left and
//   right of i hold a more valuable prize. Find the diamond, ideally in <= 5000 asks.
// Solution: lollipops are the vast majority, so asking the first 450 boxes meets one and the
//   largest left + right seen is the count of non-lollipops. Divide and conquer on [l, r]
//   knowing the non-lollipops left of each end: walk left from mid past non-lollipops to a
//   lollipop c, then recurse on [l, c - 1] and [mid + 1, r] while their counts differ; a
//   sum of 0 is the diamond. About sqrt(n) log n asks.
// Stdin: n, then hidden lollipop types t[0..n] (unique 0 = diamond; smaller = rarer).
// Uses top-level recursive fn (working around cpl-emit recursive nested-fn self-capture bug).
// original: corpus/submissions/5659145.cpp

fn ri() -> Int:
    let x: Int = read()
    return x

var g_n = 0
var g_kind: List<Int> = []
var g_seen: List<Bool> = []
var g_ml: List<Int> = []
var g_mr: List<Int> = []
var g_max_sum = 0

fn query(i: Int) -> (Int, Int):
    if not g_seen[i]:
        g_seen[i] = true
        var l = 0
        var r = 0
        for j in 0..i:
            if g_kind[j] < g_kind[i]: l += 1
        for j in i + 1..g_n:
            if g_kind[j] < g_kind[i]: r += 1
        g_ml[i] = l
        g_mr[i] = r
        g_max_sum max= l + r
    return (g_ml[i], g_mr[i])

fn search(l: Int, r: Int, lo_cnt: Int, hi_cnt: Int) -> Int:
    if l > r or lo_cnt == hi_cnt: return -1
    let mid = (l + r) / 2
    var c = mid
    while c >= l:
        let res = query(c)
        if res.0 + res.1 == 0: return c
        if res.0 + res.1 == g_max_sum:
            let left = search(l, c - 1, lo_cnt, res.0)
            if left != -1: return left
            return search(mid + 1, r, res.0 + (mid - c), hi_cnt)
        c -= 1
    return search(mid + 1, r, lo_cnt + (mid - l + 1), hi_cnt)

g_n = ri()
for _ in 0..g_n: g_kind.push(ri())
g_seen = [0..g_n] _ => false
g_ml = [0..g_n] _ => 0
g_mr = [0..g_n] _ => 0

let probe = min(g_n, 450)
var found = -1
for i in 0..probe:
    let res = query(i)
    if res.0 + res.1 == 0:
        found = i
        break
if found == -1: found = search(0, g_n - 1, 0, g_max_sum)
print(found)

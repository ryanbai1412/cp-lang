// Source: CCO 2018 Day 2 P3, Flop Sorting, https://dmoj.ca/problem/cco18p6
// limits: 1000 ms, 1024 MB
// Problem: two permutations a and b of 1..=N (N <= 4096). A flop on l..=r swaps the minimum
//   and maximum of a_l..a_r. Print at most 3e5 flops that turn a into b.
// Solution: a flop on l..=r swaps its ends when they are its min and max, so a monotone run
//   is reversed by len/2 flops. Merge sort both a and b: merge two sorted halves by rotating
//   the left half's elements >= the median of l..r past the right half's elements below it
//   (three run reversals), then recurse on the two sides. Print a's flops, then b's in reverse
//   (each flop undoes itself). O(N log^2 N) flops.
// original: corpus/submissions/4575245.cpp
// Bounds: n: 1..=4096
let n: 1..=4096 = read()
var a0: List<Int> = []
for _ in 0..n:
    let x: Int = read()
    a0.push(x - 1)
var b0: List<Int> = []
for _ in 0..n:
    let x: Int = read()
    b0.push(x - 1)

var cur: List<Int> = []
var ox: List<Int> = []
var oy: List<Int> = []

fn flip(l0: Int, r0: Int):
    var l = l0
    var r = r0 - 1
    while l < r:
        cur.swap(l, r)
        ox.push(l)
        oy.push(r)
        l += 1
        r -= 1

fn lb(l0: Int, r0: Int, target: Int) -> Int:
    let lo = (l0..r0).first_true(mid => cur[mid] >= target) ?? r0
    lo

fn merge(l: Int, mid: Int, r: Int):
    if l == mid or mid == r: return
    var sub: List<Int> = [cur[i] for i in l..r]
    sub.sort()
    let med = sub[sub.len / 2]
    let m1 = lb(l, mid, med)
    let m2 = lb(mid, r, med)
    flip(m1, mid)
    flip(mid, m2)
    flip(m1, m2)
    let len1 = m1 - l
    let len2 = mid - m1
    let len3 = m2 - mid
    let len4 = r - m2
    merge(l, l + len1, l + len1 + len3)
    merge(r - len4 - len2, r - len4, r)

fn dnc(l: Int, r: Int):
    if r - l <= 1: return
    let mid = (l + r) / 2
    dnc(l, mid)
    dnc(mid, r)
    merge(l, mid, r)

for i in 0..n: cur.push(a0[i])
dnc(0, n)
var fx: List<Int> = []
var fy: List<Int> = []
for i in 0..ox.len:
    fx.push(ox[i])
    fy.push(oy[i])

cur.clear()
ox.clear()
oy.clear()
for i in 0..n: cur.push(b0[i])
dnc(0, n)

let total = fx.len + ox.len
print(total)
for i in 0..fx.len: print(fx[i] + 1, fy[i] + 1)
var ri = ox.len - 1
while ri >= 0:
    print(ox[ri] + 1, oy[ri] + 1)
    ri -= 1

// Source: Codeforces 2157C (Meximum Array 2), https://codeforces.com/contest/2157/problem/C
// limits: 2000 ms, 256 MB
// Problem: n, k <= 100 and q <= 100 constraints (c, l, r): c = 1 means min(a[l..=r]) = k, c = 2
//   means mex(a[l..=r]) = k. Print any array of values in 0..=1e9 meeting all of them (one is
//   guaranteed to exist). Multitest.
// Solution: mark the positions covered by some min range and by some mex range, O(nq). A
//   min-only or free position gets k; one in both gets k + 1 (>= k and != k); the mex-only
//   positions, left to right, cycle through 0..k-1, so every mex range holds all of 0..k-1 and
//   no k.
// Bounds: n, k, q: 1..=100

enum Kind:
    Min
    Mex

@!multitest
let (n, k, q): (1..=100, 1..=100, 1..=100) = read()
domain Pos = 1..=n
var in_min: [Pos]Bool = false
var in_mex: [Pos]Bool = false

for _ in 0..q:
    let (c, l, r): (1..=2, Pos, Pos) = read()
    let kind = if c == 1: Kind.Min else: Kind.Mex
    for i in l..=r:
        match kind:
            Kind.Min => in_min[i] = true
            Kind.Mex => in_mex[i] = true

var a: [Pos]Int = k
var cnt = 0
for i in Pos if in_mex[i]:
    if in_min[i]:
        a[i] = k + 1
    else:
        a[i] = cnt % k
        cnt += 1
print(a)

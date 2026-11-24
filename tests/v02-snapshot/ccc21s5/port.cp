// Source: CCC 2021 S5 (Math Homework), https://dmoj.ca/problem/ccc21s5
// Problem: n <= 150000 and m <= 150000 requirements gcd(A[x..=y]) = z with z <= 16. Print
//   any A with 1 <= A_i <= 1e9 meeting all of them, or "Impossible".
// Solution: A_i must be a multiple of every z whose range covers i, and the lcm of those z
//   (at most lcm(1..16) = 720720) is the smallest choice, so if it fails nothing works. Per
//   z, a difference array over the cuts marks the covered positions (O(16 n)); then a gcd
//   SegTree checks every requirement, O(m log n).
let (n, m): (1..=150_000, 1..=150_000) = read()
domain I = 1..=n
let reqs: [0..m](l: I, r: I, x: 1..=16) = read()

// delta[x, b]: change at boundary b in how many requirements with value x are open
var delta: [0..=16, I.cuts]Int = 0
for (l, r, x) in reqs:
    delta[x, l.l] += 1
    delta[x, r.r] -= 1

var a: [I]Int = 1
for x in 1..=16:
    var open = 0
    for i in I:
        open += delta[x, i.l]
        if open > 0: a[i] = lcm(a[i], x)

let gcds = SegTree<I, Int>(gcd, a)
if all(gcds.fold(l..=r) == x for (l, r, x) in reqs): print(a) else: print("Impossible")

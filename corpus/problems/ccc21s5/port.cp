// Source: CCC 2021 S5 (Math Homework), https://dmoj.ca/problem/ccc21s5
// limits: 3000 ms, 1024 MB
// Problem: n <= 150000 and m <= 150000 requirements gcd(A[x..=y]) = z with z <= 16. Print
//   any A with 1 <= A_i <= 1e9 meeting all of them, or "Impossible".
// Solution: A_i must be a multiple of every z whose range covers i, and the lcm of those z
//   (at most lcm(1..16) = 720720) is the smallest choice, so if it fails nothing works. Per
//   z, a difference array over the cuts marks the covered positions (O(16 n)); then a gcd
//   SegTree checks every requirement, O(m log n).
// original: corpus/submissions/4323513.cpp
// Bounds: n, m: 1..=150_000; reqs.l, reqs.r: 1..=n; reqs.z: 1..=16
let (n, m): (1..=150_000, 1..=150_000) = read()
domain I = 1..=n
let reqs: [m](l: I, r: I, z: 1..=16) = read()

fn covered(z: 1..=16) -> [I]Int:
    var d = Diff<I, Int>()
    for q in reqs if q.z == z: d.add(q.l..=q.r, 1)
    d.finish()

var a: [I]Int = 1
for z in 1..=16:
    let c = covered(z)
    for i in I if c[i] > 0: a[i] = lcm(a[i], z)

let gcds = SegTree<I, Int>(gcd, a)
if reqs.all(q => gcds.fold(q.l..=q.r) == q.z): print(a) else: print("Impossible")

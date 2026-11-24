// Source: SAC '22 Code Challenge 2 P5 (Even Colouring), https://dmoj.ca/problem/sac22cc2p5
// Problem: an array of N <= 1e6 values (|A_i| <= 1e9) and Q <= 1e6 queries: "1 i j" sets
//   A_i = j; "2 L R" prints A_L + A_{L+2} + A_{L+4} + ..., every second element of [L, R].
// Solution: the summed indices all share L's parity, so keep one Fenwick tree per parity,
//   each holding only the entries of its parity (0 elsewhere). An update sets the entry in
//   its index's tree; a query is the range sum over [L, R] in L's tree. O((N + Q) log N).
// original: corpus/submissions/4121099.cpp
// Bounds: n, q: 1..=1_000_000; a: -1e9..=1e9; ty: 1..=2; i, l, r: 1..=n; x: -1e9..=1e9

let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain Pos = 1..=n
domain Parity = 0..2
let a: [Pos]-1e9..=1e9 = read()

fn side(i: Pos) -> Parity = Parity(i.ord % 2)

var sums: [Parity]Fenwick<Pos, Int> = [Parity] s => Fenwick([Pos] i => if side(i) == s: a[i] else: 0)
for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: Pos = read()
        let x: -1e9..=1e9 = read()
        sums[side(i)].set(i, x)
    else:
        let (l, r): (Pos, Pos) = read()
        print(sums[side(l)].fold(l..=r))

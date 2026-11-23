// Source: SAC '22 Code Challenge 2 P5 (Even Colouring), https://dmoj.ca/problem/sac22cc2p5
// Problem: an array of N <= 1e6 values (|A_i| <= 1e9) and Q <= 1e6 queries: "1 i j" sets
//   A_i = j; "2 L R" prints A_L + A_{L+2} + A_{L+4} + ..., every second element of [L, R].
// Solution: the summed indices all share L's parity, so keep one Fenwick tree per parity,
//   each holding only the entries of its parity. An update adds the change to its index's
//   tree; a query is the range sum over [L, R] in L's tree. O((N + Q) log N).

let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain Pos = 1..=n

// one Fenwick tree per index parity; an entry counts only in its own parity's tree
var cur: [Pos]Int = 0
var fen = [Fenwick<Pos, Int>(), Fenwick<Pos, Int>()]

for i in Pos:
    let x: -1_000_000_000..=1_000_000_000 = read()
    cur[i] = x
    fen[i.ord % 2].add(i, x)

for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: Pos = read()
        let x: -1_000_000_000..=1_000_000_000 = read()
        fen[i.ord % 2].add(i, x - cur[i])
        cur[i] = x
    else:
        let (l, r): (Pos, Pos) = read()
        print(fen[l.ord % 2].fold(l..=r))

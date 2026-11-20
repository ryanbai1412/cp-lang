// SAC '22 CC2 P5 — original: corpus/submissions/4121099.cpp
// Idiomatic CP Lang v2 (locked spec: docs/one-page-cheat-sheet.md)

let (n, q): (1..=1_000_000, 1..=1_000_000) = read()
domain Pos = 1..=n
let a: [Pos]Int = read()

@tags('1', '2')
enum Op: Set(i: Pos, x: Int) | Sum(l: Pos, r: Pos)

// one tree per parity; a position counts only in the tree of its own parity
// FRICTION 46.10: Fenwick has add but no set, so a point assignment needs SegTree
var st = [SegTree<Pos, Int>(+, [Pos] i => if i.to_idx % 2 == p: a[i] else: 0) for p in 0..2]

for _ in 0..q:
    match read::<Op>():
        Set(i, x) => st[i.to_idx % 2].set(i, x)
        Sum(l, r) => print(st[l.to_idx % 2].fold(l..=r))

// Source: NOI 2002 P1 (Galactic Heroes), https://dmoj.ca/problem/noi02p1
// Problem: 30000 ships, each its own column. T <= 5e5 lines: "M i j" moves i's whole column
//   behind j's (they differ); "C i j" prints how many ships stand between i and j, or -1 if
//   they are in different columns.
// Solution: weighted union-find: each ship keeps its distance to its parent and each root its
//   column's size; a merge hangs i's root under j's at distance size(j). Path compression adds
//   up distances, so a query is |d_i - d_j| - 1. O((T + n) alpha(n)).
// original: corpus/submissions/4793930.cpp
// Bounds: t: 1..=500_000
domain Ship = 1..=30000
enum Cmd = M(i: Ship, j: Ship) | C(i: Ship, j: Ship)
var line = PotentialDsu<Ship>()
var head: [Ship]?Ship = None
fn front(v: Ship) -> Ship:
    let r = line.find(v).root
    head[r] ?? r
let t = read<1..=500_000>()
for cmd in read<[t]Cmd>():
    match cmd:
        M(i, j) =>
            let (fi, fj) = (front(i), front(j))
            line.unite(fj, fi, line.size(j))
            head[line.find(fi).root] = Some(fj)
        C(i, j) => print(line.diff(i, j).map(d => abs(d) - 1) ?? -1)

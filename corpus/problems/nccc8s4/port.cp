// Source: Mock CCC '21 S4 (Infinite Molecule), https://dmoj.ca/problem/nccc8s4
// Problem: N <= 4e4 square atom types, each with four side labels in -26..=26 (clockwise).
//   Atoms join along a side whose two labels are nonzero and sum to 0; they may be rotated and
//   reflected, with unlimited copies. Print 1 if an infinite molecule can be built, else 0.
// Solution: an infinite molecule holds an infinite chain, entering each atom by one side and
//   leaving by another. Make a graph on the 53 labels: an atom with nonzero sides i != j adds
//   the edge tile[i] -> -tile[j] (the next atom needs a side labelled -tile[j]). The molecule
//   exists iff this graph has a cycle: a coloured DFS, O(16 N + 53^2).
// original: corpus/submissions/3381215.java
// Bounds: m: 1..=100_000; v: -26..=26
domain Side = 0..=52
let m: 1..=100_000 = read()
var adj: [Side, Side]Bool = false
for _ in 0..m:
    var tile: [0..4]Int = 0
    for j in 0..4:
        let v: -26..=26 = read()
        tile[j] = v
    for i in 0..4:
        if tile[i] != 0:
            for j in 0..4:
                if j != i and tile[j] != 0:
                    adj[Side.at(tile[i] + 26)!, Side.at(26 - tile[j])!] = true

var state: [Side]Int = 0
var has_cycle = false
fn dfs(u: Side):
    if state[u] != 0:
        if state[u] == 1: has_cycle = true
        return
    state[u] = 1
    for v in Side:
        if adj[u, v]: dfs(v)
    state[u] = 2

for u in Side: dfs(u)
if has_cycle: print(1)
else: print(0)

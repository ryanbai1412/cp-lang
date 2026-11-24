// Source: Mock CCC '21 S4 (Infinite Molecule), https://dmoj.ca/problem/nccc8s4
// Problem: N <= 4e4 square atom types, each with four side labels in -26..=26 (clockwise).
//   Atoms join along a side whose two labels are nonzero and sum to 0; they may be rotated and
//   reflected, with unlimited copies. Print 1 if an infinite molecule can be built, else 0.
// Solution: an infinite molecule holds an infinite chain, entering each atom by one side and
//   leaving by another. Make a graph on the 53 labels: an atom with nonzero sides i != j adds
//   the edge tile[i] -> -tile[j] (the next atom needs a side labelled -tile[j]). The molecule
//   exists iff this graph has a cycle: std `find_cycle`, O(16 N + 53^2).
// Bounds: m: 1..=100_000; atoms: -26..=26
use std.graph.{find_cycle}

domain Label = -26..=26
let m: 1..=100_000 = read()
domain Atom = 0..m
domain Side = 0..4
let atoms: [Atom, Side]Label = read()

var joins: [Label, Label]Bool = false
for a in Atom:
    for i in Side:
        for j in Side:
            if i != j and atoms[a, i] != 0 and atoms[a, j] != 0:
                joins[atoms[a, i], Label(-Int(atoms[a, j]))] = true
let next: [Label]List<Label> = [Label] u => [v for v in Label if joins[u, v]]
print(if find_cycle(next) == None: 0 else: 1)

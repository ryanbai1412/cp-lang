// Source: CCC 2018 S2 (Sunflowers), https://dmoj.ca/problem/ccc18s2
// Problem: an n x n grid, 2 <= n <= 100, of positive values <= 1e9 is a rotation by a multiple of
//   90 degrees of a table whose rows increase left to right and columns top to bottom. Print the
//   original table.
// Solution: only the original orientation increases both ways: try up to four clockwise rotations,
//   checking every row and column, and print the first that passes. O(n^2).
// original: corpus/submissions/4329456.cpp
// Bounds: n: 2..=100; given: 1..=1e9
let n: 2..=100 = read()
domain I = 0..n
let given: [I, I]1..=1e9 = read()

fn turned(g: [I, I]Int) -> [I, I]Int = [I, I] (r, c) => g[I(n - 1 - c.ord), r]
fn increasing(line: [I]Int) -> Bool = line.adjacent.all((a, b) => a < b)
fn grows(g: [I, I]Int) -> Bool = all(increasing(g[r]) for r in I) and all(increasing(g[.., c]) for c in I)

var g = given
while not grows(g): g = turned(g)
for r in I: print(g[r])

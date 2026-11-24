// Source: CCC 2020 S2 (Escape Room, hard data), https://dmoj.ca/problem/ccc20s2hard
// limits: 3000 ms, 512 MB
// Problem: an M x N grid, M, N <= 4000, of values 1..MN. From a cell holding x you may jump to any
//   cell (a, b) with a * b = x. Starting at (1, 1), can you reach (M, N)? The judge calls
//   can_escape(M, N, v); like the original, this reads M, N and the (M + 1) x (N + 1) array v
//   (row and column 0 all zeros) and prints yes or no.
// Solution: where you can jump depends only on the value held, so the nodes are values: cell
//   (r, c) is an edge from value r * c to the value it holds. Group those arcs by source in a CSR
//   array (a counting sort) and walk from the value of (1, 1); holding MN means (M, N) is reachable. O(MN).
// original: corpus/submissions/4324075.cpp
with io(answers: ("yes", "no"))
let rows: 1..=4000 = read()
let cols: 1..=4000 = read()
domain R = 0..=rows  // row and column 0 are zeros, as the judge passes them
domain C = 0..=cols
domain Num = 1..=rows * cols
let room: [R, C]0..=rows * cols = read()

// The arcs grouped by their source value, as CSR: value x's targets are to[at[x]..at[x] + deg].
// (std `Graph.of` would buffer every (from, arc) pair first, about 50% more memory here.)
var at: [Num]Int = 0
for r in R.all.drop(1):
    for c in C.all.drop(1): at[Num(r.ord * c.ord)] += 1
var used = 0
for x in Num: (at[x], used) = (used, used + at[x])
var to = List.filled(rows * cols, Num.first!)
for r in R.all.drop(1):
    for c in C.all.drop(1):
        let x = Num(r.ord * c.ord)
        to[at[x]] = Num(room[r, c])
        at[x] += 1
// Now at[x] is the end of x's arcs, which start where x - 1's end.
fn leads(x: Num) -> yields Num:
    for i in (if let p = x.prev: at[p] else: 0)..at[x]: yield to[i]
print(yes(reachable(leads, from: [Num(room[R(1), C(1)])]).any(x => x == Num.last!)))

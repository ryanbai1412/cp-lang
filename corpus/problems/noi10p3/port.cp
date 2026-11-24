// Source: NOI 2010 P3 (Altitude), https://dmoj.ca/problem/noi10p3
// Problem: an (n+1) x (n+1) lattice of junctions, n <= 500; every street segment has a flow up
//   to 1e6 in each direction (input: the east, south, west, north flows). Heights are 0 at the
//   top-left and 1 at the bottom-right; climbing by h costs flow * h. Choose the other heights to
//   minimise the total cost and print it, rounded.
// Solution: some optimum uses only heights 0 and 1, so the answer is a minimum cut between the
//   two corners, found as a shortest path in the planar dual: one vertex per cell plus the
//   outside split into the low side (left and bottom) and the high side (top and right). Each
//   segment's flow is paid when the cut has its tail at 0 and its head at 1, so it becomes one
//   directed dual arc across it. Dijkstra from low to high, O(n^2 log n).
// original: corpus/submissions/5672180.cpp
// Bounds: n: 1..=500; east, south, west, north: 0..=1e6

let n: 1..=500 = read()
let east: [0..=n, 1..=n]0..=1e6 = read()
let south: [1..=n, 0..=n]0..=1e6 = read()
let west: [0..=n, 1..=n]0..=1e6 = read()
let north: [1..=n, 0..=n]0..=1e6 = read()

domain Face = 0..n * n + 2
let (low, high) = (Face(0), Face(n * n + 1))
fn cell(i: 1..=n, j: 1..=n) -> Face = Face((i - 1) * n + j)
// the faces on each side of the segment on row line i after column j, or on column line j in row i
fn above(i: 0..=n, j: 1..=n) -> Face = if i == 0: high else: cell(i, j)
fn below(i: 0..=n, j: 1..=n) -> Face = if i == n: low else: cell(i + 1, j)
fn left(i: 1..=n, j: 0..=n) -> Face = if j == 0: low else: cell(i, j)
fn right(i: 1..=n, j: 0..=n) -> Face = if j == n: high else: cell(i, j + 1)

var dual: List<(u: Face, v: Face, w: Int)> = []
for i in 0..=n:
    for j in 1..=n:
        dual.push((u: below(i, j), v: above(i, j), w: east[i, j]))
        dual.push((u: above(i, j), v: below(i, j), w: west[i, j]))
for i in 1..=n:
    for j in 0..=n:
        dual.push((u: left(i, j), v: right(i, j), w: south[i, j]))
        dual.push((u: right(i, j), v: left(i, j), w: north[i, j]))
domain Arc = 0..dual.len
print(dijkstra(Graph.directed([Arc] a => dual[a.ord]), from: [(low, 0)], until: high)[high]!)

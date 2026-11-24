// Source: IOI 2012 Day 2 (Ideal City), https://dmoj.ca/problem/ioi12p4
// limits: 1000 ms, 256 MB
// Problem: N <= 1e5 blocks on grid cells (coordinates below 2^31) form an ideal city: the
//   blocks are connected and so are the empty cells (no holes). The distance of two blocks is
//   the fewest hops through adjacent blocks. Print the sum over all pairs, modulo 1e9.
// Solution: count steps along each axis separately. Cut every column into runs of
//   touching blocks; without holes, runs that touch side by side form a tree, and a
//   shortest path crosses exactly the tree edges between its ends' runs. So an edge with w
//   blocks on one side is crossed by w * (N - w) pairs; do the same with the axes swapped:
//   O(N log N) for the sort and the run map.
// Bounds: n: 1..=1e5; at.x, at.y: 0..=2147483647
// original: corpus/problems/ioi12p4/orig.cpp (corpus/submissions/4951048.cpp with a main)
use std.graph.{undirected_of_pairs}

let n: 1..=1e5 = read()
domain Block = 0..n
// "connected, no holes" is not a type; gen.py builds such cities
type Coord = 0..=2147483647  // below 2^31
let at: [Block](x: Coord, y: Coord) = read()

// Steps east or west, summed over all pairs. Cut each street (one x) into
// runs of touching blocks. Without holes the runs form a tree, and a shortest
// way between two blocks crosses exactly the tree edges between their runs.
// So an edge with w blocks on one side is crossed by w * (n - w) pairs.
fn crossings(blocks: List<(Int, Int)>) -> Int:
    var run_of: Map<(Int, Int), Int> = []  // a HashMap keyed by a pair builds, then fails in g++
    var size: List<Int> = []
    var last: ?(Int, Int) = None
    for (x, y) in blocks.sorted():
        if last != Some((x, y - 1)): size.push(0)
        size.last! += 1
        run_of[(x, y)] = size.len - 1
        last = Some((x, y))

    domain Run = 0..size.len
    // one edge per pair of touching runs: skip a block when the blocks just
    // below it already made the edge
    fn starts_edge(x: Int, y: Int) -> Bool =
        (x - 1, y) in run_of and not ((x, y - 1) in run_of and (x - 1, y - 1) in run_of)
    let edges = [(Run(run_of[(x - 1, y)]), Run(run_of[(x, y)])) for (x, y) in blocks if starts_edge(x, y)]
    let tree = Rooted(undirected_of_pairs(edges), Run.first!)
    var under: [Run]Int = [Run] r => size[r.label]
    var total = 0
    for r in tree.at.rev:
        if let p = tree.parent(r):
            total += under[r] * (n - under[r])
            under[p] += under[r]
    total

print((crossings([(b.x, b.y) for b in at]) + crossings([(b.y, b.x) for b in at])) % 1e9)

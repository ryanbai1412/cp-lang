// Source: NOI 2016 Day 1 P2 (Grid), https://dmoj.ca/problem/noi16p2
// limits: 2000 ms, 1024 MB
// Problem: multitest, T <= 20. An n x m grid (n, m <= 1e9) has c crickets in given cells (sum
//   of c <= 1e5) and fleas everywhere else, linked by side adjacency. Print the least number of
//   fleas to turn into crickets so that some two fleas are disconnected, or -1 if impossible.
// Solution: the answer is -1, 0, 1 or 2. 0 if the fleas are already split: top to bottom, each
//   row holding crickets breaks into free spans and each band of empty rows is one full-width
//   span; a Dsu joins the overlapping spans of neighbouring bands, and the fleas are split iff
//   more than one set remains. -1 if at most 2 fleas remain. 1 if some cut vertex of the graph
//   of free cells within distance 2 (3 on a 1-wide grid) of a cricket touches a cricket or the
//   border 8-way; with no crickets, iff n or m is 1. Otherwise 2. O(c log c) per test.
// original: corpus/submissions/4798070.cpp
// Bounds: t: 1..=20; n, m: 1..=1e9; c: 0..=100_000; x: 1..=n; y: 1..=m

use std.graph.{cut_vertices}

type Cell = (x: Int, y: Int)
struct Span(l: Int, r: Int)

/// Whether the fleas already fall apart.
fn split(n: Int, m: Int, crickets: List<Cell>) -> Bool:
    // the free spans of each band, top to bottom: a row with crickets, or a run of empty rows
    var bands: List<List<Span>> = []
    var above = 0
    for row in crickets.runs(by: p => p.x):
        let x = row.first!.x
        if x > above + 1: bands.push([Span(1, m)])
        let walls = [0] + [p.y for p in row] + [m + 1]
        bands.push([Span(a + 1, b - 1) for (a, b) in walls.adjacent if b > a + 1])
        above = x
    if n > above: bands.push([Span(1, m)])

    domain Run = 0..bands.map(b => b.len).sum()
    var joined = Dsu<Run>()
    var top = 0  // the first run of the band above
    for (up, down) in bands.adjacent:
        let bottom = top + up.len
        var (i, j) = (0, 0)
        while i < up.len and j < down.len:
            if up[i].l <= down[j].r and down[j].l <= up[i].r: joined.unite(Run(top + i), Run(bottom + j))
            if up[i].r < down[j].r: i += 1 else: j += 1
        top = bottom
    joined.components > 1

/// Whether one more cricket can split the fleas.
fn one_cut(n: Int, m: Int, crickets: List<Cell>) -> Bool:
    if crickets.is_empty: return n == 1 or m == 1
    fn on_grid(p: Cell) -> Bool = 1 <= p.x and p.x <= n and 1 <= p.y and p.y <= m
    let (_, cricket) = compress(crickets)
    let r = if min(n, m) == 1: 3 else: 2
    var nearby: List<Cell> = []
    for p in crickets:
        for dx in -r..=r:
            for dy in -r..=r:
                let q = (x: p.x + dx, y: p.y + dy)
                if on_grid(q) and not cricket.contains(q): nearby.push(q)
    let (domain Near, near) = compress(nearby)
    var links: List<(u: Near, v: Near)> = []
    for (u, p) in near.items:
        for q in [(x: p.x + 1, y: p.y), (x: p.x, y: p.y + 1)]:
            if let v = near.try_index_of(q): links.push((u:, v:))
    domain Link = 0..links.len
    let link: [Link](u: Near, v: Near) = [Link] e => links[e.ord]
    let cut = cut_vertices(Graph.undirected(link))

    fn walled(p: Cell) -> Bool:
        for dx in -1..=1:
            for dy in -1..=1:
                let q = (x: p.x + dx, y: p.y + dy)
                if q != p and (not on_grid(q) or cricket.contains(q)): return true
        false
    Near.any(u => cut[u] and walled(near.get(u)))

let t: 1..=20 = read()
repeat t:
    let (n, m, c): (1..=1e9, 1..=1e9, 0..=100_000) = read()
    let crickets: List<Cell> = read<[c](x: 1..=n, y: 1..=m)>().sorted()
    print(if split(n, m, crickets): 0 elif n * m - c <= 2: -1 elif one_cut(n, m, crickets): 1 else: 2)

// CCC '20 S2 (Hard) — Escape Room (Hard) (DMOJ: ccc20s2hard)
// original: corpus/submissions/4324075.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Reachability from (1, 1) to (n, m) in an n x m grid (n, m <= 4000, up to 16M
// cells) where from any cell with value x you can jump to any cell (r, c) with
// r * c == x. On DMOJ, ccc20s2hard uses a function-style grader interface and a
// tight memory limit that forced C++ to hand-roll a flat CSR arena + bitset.
//
// In CP Lang v2, `@grader` exports `can_escape`, `Graph.directed` compiles to
// flat CSR with zero per-node vector heap headers, and `Bitset<CellVal>` packs
// visited flags into 2 MB (16M bits).
@grader
fn can_escape(n: 1..=4000, m: 1..=4000, grid: [0..=n, 0..=m]Int) -> Bool:
    let target = n * m
    domain Val = 1..=target
    var edges: List<(Val, Val)> = []
    for r in 1..=n:
        for c in 1..=m:
            let v = grid[r, c]
            if v in Val:
                edges.push((Val(r * c), Val(v)))

    let g = Graph.directed(edges)
    let start = grid[1, 1]
    if start not in Val: return false

    var vis = Bitset<Val>()
    var stk: List<Val> = [Val(start)]
    vis.add(Val(start))
    while let u = stk.pop():
        if u == Val.last: return true
        for v in g[u] if v not in vis:
            vis.add(v)
            stk.push(v)
    false

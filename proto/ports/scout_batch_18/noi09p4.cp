// NOI '09 P4 - Plants vs. Zombies (DMOJ: noi09p4, 25pt, TL 1.0s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Grid of n rows (0..n-1) by m columns (0..m-1). Cell (r, c) has score s
// (may be negative) and protects w other cells. Zombies enter from the right
// (column m - 1), so (r, c + 1) must be eaten before (r, c); also every
// protector u must be eaten before its protected cell v.
// Maximize the total score of a valid subset of eaten cells.
//
// 1) Build dependency graph `dep`: u -> v means u must be eaten before v.
// 2) Run Kahn's BFS from in-degree 0 to keep only cells not in or behind a cycle.
// 3) Maximum-weight closure via Dinic Max-Flow (`FlowGraph`):
//    - pos_sum = sum of positive scores among reachable cells
//    - src -> v (cap score[v]) if score[v] > 0; v -> sink (cap -score[v]) if < 0
//    - for every dep edge u -> v (both reachable), add v -> u with cap INF.
//    Answer = pos_sum - max_flow(src, sink).
use std.flow.{FlowGraph}

let (n, m): (1..=20, 1..=30) = read()
domain Row = 0..n
domain Col = 0..m
// WISH W15: variable-length per-cell protection list cannot be read as one typed
// `[Row, Col](score: Int, prot: List<(Row, Col)>) = read()` because `w` precedes the list.
var score: [Row, Col]Int = 0
var dep: [Row, Col]List<(Row, Col)> = []
var indeg: [Row, Col]Int = 0

for r in Row:
    for c in Col:
        score[r, c] = read()
        let w: 0..=600 = read()
        for _ in 0..w:
            let (pr, pc): (Row, Col) = read()
            dep[r, c].push((pr, pc))
            indeg[pr, pc] += 1
        if let right = c + 1:
            dep[r, right].push((r, c))
            indeg[r, c] += 1

fn solve() -> Int:
    // WISH W16: `stdlib/graph.cp`'s `topo` returns `none` on any cycle instead of
    // returning the acyclic reachable prefix (cells whose prerequisites are cycle-free).
    var can_eat: [Row, Col]Bool = false
    var deg = indeg.copy()
    var queue: List<(Row, Col)> = []
    for r in Row:
        for c in Col:
            if deg[r, c] == 0:
                can_eat[r, c] = true
                queue.push((r, c))
    var head = 0
    while head < queue.len:
        let (r, c) = queue[head]
        head += 1
        for (nr, nc) in dep[r, c]:
            deg[nr, nc] -= 1
            if deg[nr, nc] == 0:
                can_eat[nr, nc] = true
                queue.push((nr, nc))

    // WISH W17: Multi-feature collision — `FlowGraph<Node>` cannot be parameterized
    // by a tagged union `Cell | Src | Sink`, forcing manual flattening of (Row, Col)
    // into `0..(n * m + 2)` and losing product-domain safety.
    domain Node = 0..(n * m + 2)
    let src = Node(n * m)
    let sink = Node(n * m + 1)
    fn id(r: Row, c: Col) -> Node = Node(r.to_idx * m + c.to_idx)

    var fg = FlowGraph<Node>()
    var pos_sum = 0
    const INF = 1_000_000_000
    for r in Row:
        for c in Col:
            if not can_eat[r, c]: continue
            let u = id(r, c)
            if score[r, c] > 0:
                pos_sum += score[r, c]
                fg.add(src, u, score[r, c])
            elif score[r, c] < 0:
                fg.add(u, sink, -score[r, c])
            for (nr, nc) in dep[r, c]:
                if can_eat[nr, nc]:
                    fg.add(id(nr, nc), u, INF)
    pos_sum - fg.max_flow(src, sink)

@brute(solve)
fn solve_brute() -> Int
    where n * m <= 12:
    let k = n * m
    var best = 0
    for mask in 0..(1 << k):
        var ok = true
        var tot = 0
        for r in Row:
            for c in Col:
                let bit_u = r.to_idx * m + c.to_idx
                let eaten_u = ((mask >> bit_u) & 1) == 1
                if eaten_u: tot += score[r, c]
                for (nr, nc) in dep[r, c]:
                    let bit_v = nr.to_idx * m + nc.to_idx
                    let eaten_v = ((mask >> bit_v) & 1) == 1
                    if eaten_v and not eaten_u: ok = false
        if ok: best max= tot
    best

print(solve())

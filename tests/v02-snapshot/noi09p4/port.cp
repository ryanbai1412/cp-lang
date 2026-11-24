// Source: NOI 2009 P4 (Plants vs. Zombies), https://dmoj.ca/problem/noi09p4
// Problem: an N x M grid of plants (N <= 20, M <= 30), each with an energy score in
//   [-10000, 10000] and a list of cells it protects. Zombies enter each row from the right and
//   must eat a row's plants right to left, and cannot eat a protected plant before its
//   protectors. Input: N M, then per plant its score, w, and w cells. Print the best total.
// Solution: "x must be eaten before y" edges (right neighbour, protectors); a topological peel
//   drops every plant on or behind a cycle. The rest is a maximum-weight closure: source -> plant
//   with positive score, plant -> sink with negative, infinite edges to prerequisites; the
//   answer is the positive sum minus the max flow (FlowGraph) on NM + 2 nodes.
// Runnable port (proto/cp.py lib-run) of statement-first noi09p4.cp
// original: corpus/submissions/5144326.cpp

let (n, m): (Int, Int) = read()
domain Row = 0..n
domain Col = 0..m
var score: [Row, Col]Int = [Row, Col] (_, _) => 0
var dep: [Row, Col]List<(r: Row, c: Col)> = [Row, Col] (_, _) => []
var indeg: [Row, Col]Int = [Row, Col] (_, _) => 0

for r in Row:
    for c in Col:
        let s: Int = read()
        score[r, c] = s
        let w: Int = read()
        for _ in 0..w:
            let (pr, pc): (Row, Col) = read()
            dep[r, c].push((r: pr, c: pc))
            indeg[pr, pc] += 1
        if c.ord + 1 < m:
            let right = Col(c.ord + 1)
            dep[r, right].push((r:, c:))
            indeg[r, c] += 1

var can_eat: [Row, Col]Bool = [Row, Col] (_, _) => false
var deg: [Row, Col]Int = [Row, Col] (r, c) => indeg[r, c]
var qr: List<Row> = []
var qc: List<Col> = []
for r in Row:
    for c in Col:
        if deg[r, c] == 0:
            can_eat[r, c] = true
            qr.push(r)
            qc.push(c)

var head = 0
while head < qr.len:
    let r = qr[head]
    let c = qc[head]
    head += 1
    for nxt in dep[r, c]:
        deg[nxt.r, nxt.c] -= 1
        if deg[nxt.r, nxt.c] == 0:
            can_eat[nxt.r, nxt.c] = true
            qr.push(nxt.r)
            qc.push(nxt.c)

domain Node = 0..(n * m + 2)
let src = Node(n * m)
let sink = Node(n * m + 1)
fn id(r: Row, c: Col) -> Node = Node(r.ord * m + c.ord)

var fg = FlowGraph<Node>.new()
var pos_sum = 0
for r in Row:
    for c in Col:
        if not can_eat[r, c]: continue
        let u = id(r, c)
        if score[r, c] > 0:
            pos_sum += score[r, c]
            fg.add(src, u, score[r, c])
        elif score[r, c] < 0:
            fg.add(u, sink, -score[r, c])
        for nxt in dep[r, c]:
            if can_eat[nxt.r, nxt.c]:
                fg.add(id(nxt.r, nxt.c), u, cap: None)

print(pos_sum - fg.max_flow(src, sink))

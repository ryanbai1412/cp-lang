// Runnable port (proto/cp.py lib-run) of statement-first noi09p4.cp
// original: corpus/submissions/5144326.cpp
use std.flow.{FlowGraph}

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
        if c.to_idx + 1 < m:
            let right = Col(c.to_idx + 1)
            dep[r, right].push((r: r, c: c))
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
fn id(r: Row, c: Col) -> Node = Node(r.to_idx * m + c.to_idx)

var fg = FlowGraph<Node>.new()
var pos_sum = 0
let INF = 1000000000
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
                fg.add(id(nxt.r, nxt.c), u, INF)

print(pos_sum - fg.max_flow(src, sink))

// NOI 2009 Plants vs Zombies: peel the protection graph, then take a
// max-weight closure of what is left with a min cut.
// original: corpus/submissions/5144326.cpp

let (n, m): (Int, Int) = read()
domain Cell = 0..n * m
fn cell(r: Int, c: Int) -> Cell = Cell(r * m + c)

var score: [Cell]Int = 0
var protects: [Cell]List<Cell> = []
for r in 0..n:
    for c in 0..m:
        let u = cell(r, c)
        if c + 1 < m: protects[cell(r, c + 1)].push(u)
        let (value, k): (Int, Int) = read()
        score[u] = value
        for _ in 0..k:
            let (x, y): (Int, Int) = read()
            protects[u].push(cell(x, y))

var eatable: [Cell]Bool = false
for u in peel(v => protects[v]): eatable[u] = true

domain Node = 0..n * m + 2
fn node(u: Cell) -> Node = Node(u.to_idx)
let (source, sink) = (Node(n * m), Node(n * m + 1))
var net = FlowGraph<Node>()
var gain = 0
for u in Cell if eatable[u]:
    if score[u] > 0:
        gain += score[u]
        net.add(source, node(u), score[u])
    elif score[u] < 0:
        net.add(node(u), sink, -score[u])
    for v in protects[u] if eatable[v]:
        net.add(node(v), node(u), cap: none)

print(gain - net.max_flow(source, sink))

// NOI 2009 Plants vs Zombies (CP Lang 0.2.2): reverse Kahn cycle peel via
// `Deque<Cell>` + `Marks<Cell>` followed by max-weight closure via `FlowGraph`.
// original: corpus/submissions/5144326.cpp

let (n, m): (Int, Int) = read()
let total_cells = n * m
domain Cell = 0..total_cells

var score: [Cell]Int = 0
var deps: [Cell]List<Cell> = []
var rev: [Cell]List<Cell> = []
var need: [Cell]Int = 0

fn cell(r: Int, c: Int) -> Cell = Cell(r * m + c)

fn require(u: Cell, before: Cell):
    deps[u].push(before)
    rev[before].push(u)
    need[u] += 1

for r in 0..n:
    for c in 0..m:
        let u = cell(r, c)
        if c + 1 < m: require(u, cell(r, c + 1))
        let (val, k): (Int, Int) = read()
        score[u] = val
        for _ in 0..k:
            let (x, y): (Int, Int) = read()
            require(cell(x, y), u)

var eatable = Marks<Cell>()
var q = Deque<Cell>.new()
for u in Cell:
    if need[u] == 0: q.push(u)
while let u = q.pop_front():
    eatable.add(u)
    for v in rev[u]:
        need[v] -= 1
        if need[v] == 0: q.push(v)

domain Node = 0..total_cells + 2
let (src, snk) = (Node(total_cells), Node(total_cells + 1))
var net = FlowGraph<Node>()
var positive = 0
for u in Cell:
    if u not in eatable: continue
    let nu = Node(u.idx)
    if score[u] > 0:
        positive += score[u]
        net.add(src, nu, score[u])
    elif score[u] < 0:
        net.add(nu, snk, -score[u])
    for v in deps[u]:
        if v in eatable: net.add(nu, Node(v.idx), cap: none)

print(positive - net.max_flow(src, snk))

// NOI 2009 Plants vs Zombies: an n x m lawn where eating plant (r, c) scores
// a[r, c] (possibly negative). Zombies enter from the right (so (r, c + 1)
// must be eaten before (r, c)), and each plant protects a listed set of cells
// (so the protector must be eaten before any cell it protects). Find the
// maximum total score of an eatable subset of plants.
//
// Any plant that depends on a cycle (directly or transitively) can never be
// eaten. Peeling out-degree-0 vertices in the dependency graph (Kahn's order
// on reversed dependencies) finds every eatable plant in O(V + E), replacing
// the original's O(V(V + E)) repeated DFS. On the remaining DAG, choosing a
// maximum-weight closed subset reduces to s-t min-cut via `std.flow.FlowGraph`.
// original: corpus/submissions/5144326.cpp
use std.flow.{FlowGraph}

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

// A plant is eatable iff every dependency chain from it terminates (no cycle).
var eatable: [Cell]Bool = false
var q = [u for u in Cell if need[u] == 0]
var head = 0
while head < q.len:
    let u = q[head]
    head += 1
    eatable[u] = true
    for v in rev[u]:
        need[v] -= 1
        if need[v] == 0: q.push(v)

// Max-weight closure on eatable plants: flatten `Src + Cell + Snk` into one domain.
domain Node = 0..total_cells + 2
let (src, snk) = (Node(total_cells), Node(total_cells + 1))
const INF = 1000000000000
var net = FlowGraph<Node>.new()
var positive = 0
for u in Cell:
    if not eatable[u]: continue
    let nu = Node(u.to_idx)
    if score[u] > 0:
        positive += score[u]
        net.add(src, nu, score[u])
    elif score[u] < 0:
        net.add(nu, snk, -score[u])
    for v in deps[u]:
        if eatable[v]: net.add(nu, Node(v.to_idx), INF)

print(positive - net.max_flow(src, snk))

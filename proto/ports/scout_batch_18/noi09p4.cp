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
// 2) `peel` keeps the cells not on or behind a cycle.
// 3) Eating a cell needs every eatable cell before it, so the answer is the
//    maximum closure of the needs graph over the eatable cells.
use std.flow.{max_closure}

let (n, m): (1..=20, 1..=30) = read()
domain Row = 0..n
domain Col = 0..m
// WISH W17: `domain Cell = (Row, Col)` does not yet index `[Cell]T` or pass as one
// generic domain to `peel` and `max_closure`, so cells are numbered row-major by hand.
domain Cell = 0..n * m
fn cell(r: Row, c: Col) -> Cell = Cell(r.to_idx * m + c.to_idx)

// WISH W15: variable-length per-cell protection list cannot be read as one typed
// `[Row, Col](score: Int, prot: List<(Row, Col)>) = read()` because `w` precedes the list.
var score: [Cell]Int = 0
var dep: [Cell]List<Cell> = []
for r in Row:
    for c in Col:
        score[cell(r, c)] = read()
        let w: 0..=600 = read()
        for _ in 0..w:
            let (pr, pc): (Row, Col) = read()
            dep[cell(r, c)].push(cell(pr, pc))
        if let right = c + 1:
            dep[cell(r, right)].push(cell(r, c))

fn solve() -> Int:
    var can_eat: [Cell]Bool = false
    for u in peel<Cell>(u => dep[u]): can_eat[u] = true
    let profit = [Cell] u => if can_eat[u]: score[u] else: 0
    let needs = [(u: v, v: u) for u in Cell if can_eat[u] for v in dep[u] if can_eat[v]]
    max_closure(profit, needs).value

@brute(solve)
fn solve_brute() -> Int:
    assert Cell.K <= 12
    var before: [Cell]List<Cell> = []
    for u in Cell:
        for v in dep[u]: before[v].push(u)
    var best = 0
    for mask in 0..(1 << Cell.K):
        let want = [Cell] u => (mask >> u.to_idx) & 1 == 1
        var eaten: [Cell]Bool = false
        var grew = true
        while grew:
            grew = false
            for u in Cell if want[u] and not eaten[u] and all(eaten[p] for p in before[u]):
                eaten[u] = true
                grew = true
        if all(eaten[u] == want[u] for u in Cell):
            best max= sum(score[u] for u in Cell if want[u])
    best

print(solve())

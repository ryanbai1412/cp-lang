// Source: EGOI 2023 P3 (Find the Box), https://dmoj.ca/problem/egoi23p3
// limits: 1000 ms, 1024 MB
// Problem: interactive. An H x W grid (H, W <= 50) hides a solid box at some cell other than
//   (0, 0). Each query sends a move string (<= 20000 of <>^v) to a robot starting at (0, 0)
//   that stops at walls and the box, and reads where it ends. Find the box; full score needs
//   at most 2 queries.
// Solution: keep every candidate cell and simulate each query against all of them, keeping
//   those that predict the reported end. Query 1 is a snake sweep of the grid; then pick the
//   first row probe (down r rows, then right) whose simulated ends separate the survivors,
//   else a fixed fallback probe. O(H W |moves|) simulation per query.
// original: corpus/submissions/5678051.cpp
let h: 1..=100 = read()
let w: 1..=100 = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)

fn step(p: Cell, ch: Char) -> ?Cell:
    if ch == '<':
        if let c = Col.at(p.1 - 1): return (p.0, c)
    elif ch == '>':
        if let c = Col.at(p.1 + 1): return (p.0, c)
    elif ch == '^':
        if let r = Row.at(p.0 - 1): return (r, p.1)
    elif ch == 'v':
        if let r = Row.at(p.0 + 1): return (r, p.1)
    None

fn simu(bad: Cell, moves: List<Char>) -> Cell:
    var pos: Cell = (Row.first!, Col.first!)
    for ch in moves:
        if let nxt = step(pos, ch):
            if nxt != bad:
                pos = nxt
    pos

fn append_rep(mut out: List<Char>, pat: List<Char>, times: Int):
    for _ in 0..times:
        for ch in pat:
            out.push(ch)

fn sweep_moves() -> List<Char>:
    var out: List<Char> = []
    for i in 1..h:
        if i % 2 == 1:
            append_rep(mut out, ['v', '>', '^'], w - 1)
            append_rep(mut out, ['>', 'v', '>'], 1)
        else:
            append_rep(mut out, ['v', '<', '^'], w - 1)
            append_rep(mut out, ['<', 'v', '<'], 1)
    out

fn row_probe(row_idx: Int) -> List<Char>:
    var out: List<Char> = []
    append_rep(mut out, ['v'], row_idx)
    append_rep(mut out, ['>'], w)
    out

fn fallback_probe() -> List<Char>:
    var out: List<Char> = ['>', 'v', 'v', '<', '^', 'v']
    append_rep(mut out, ['>'], w)
    append_rep(mut out, ['^', 'v'], 1)
    append_rep(mut out, ['<'], w)
    append_rep(mut out, ['^', '^'], 1)
    append_rep(mut out, ['>'], w)
    out

fn separates(cands: List<Cell>, moves: List<Char>) -> Bool:
    var seen: [Row, Col]Bool = false
    for idx in 0..cands.len:
        let b = cands[idx]
        let p = simu(b, moves)
        if seen[p]: return false
        seen[p] = true
    true

fn query_filter(cands: List<Cell>, moves: List<Char>) -> List<Cell>:
    let got: (Row, Col) = ask("?", moves)
    var kept: List<Cell> = []
    for idx in 0..cands.len:
        let b = cands[idx]
        if simu(b, moves) == got:
            kept.push(b)
    kept

fn solve():
    var cands: List<Cell> = []
    for r in Row:
        for c in Col:
            if r != Row.first or c != Col.first:
                cands.push((r, c))
    if h == 1:
        var m0: List<Char> = []
        append_rep(mut m0, ['>'], w)
        let rem = query_filter(cands, m0)
        let box = rem[0]
        print("!", box.0, box.1)
        return
    let after1 = query_filter(cands, sweep_moves())
    if after1.len == 1:
        let box = after1[0]
        print("!", box.0, box.1)
        return
    for r in 0..h:
        let m2 = row_probe(r)
        if separates(after1, m2):
            let rem = query_filter(after1, m2)
            let box = rem[0]
            print("!", box.0, box.1)
            return
    let rem = query_filter(after1, fallback_probe())
    let box = rem[0]
    print("!", box.0, box.1)

solve()

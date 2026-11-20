// EGOI 2023 P3, Find the Box — CP Lang 0.2.2
// Category: Interactive (stdio ?/!, tight 2-query budget, candidate-elimination simulator)
// Original C++: corpus/submissions/5678051.cpp (154 LOC raw, ~110 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/egoi23p3.cp (verified vs orig C++)

let h: 1..=100 = read()
let w: 1..=100 = read()
assert h * w >= 2

domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)

fn step(p: Cell, ch: Char) -> ?Cell:
    match ch:
        '<' => if let c = p.1 - 1: (p.0, c) else: none
        '>' => if let c = p.1 + 1: (p.0, c) else: none
        '^' => if let r = p.0 - 1: (r, p.1) else: none
        'v' => if let r = p.0 + 1: (r, p.1) else: none
        _ => none

fn simu(box: Cell, moves: Str) -> Cell:
    var pos: Cell = (Row.first, Col.first)
    for ch in moves:
        if let nxt = step(pos, ch) and nxt != box:
            pos = nxt
    pos

fn repeat_str(s: Str, k: Int) -> Str:
    var out = ""
    for _ in 0..k: out += s
    out

fn sweep_moves() -> Str:
    var out = ""
    for i in 1..h:
        if i % 2 == 1:
            out += repeat_str("v>^", w - 1) + ">v>"
        else:
            out += repeat_str("v<^", w - 1) + "<v<"
    out

fn row_probe(r: Int) -> Str: repeat_str("v", r) + repeat_str(">", w)

fn fallback_probe() -> Str:
    ">vv<^v" + repeat_str(">", w) + "^v" + repeat_str("<", w) + "^^" + repeat_str(">", w)

fn separates(cands: List<Cell>, moves: Str) -> Bool:
    var seen: [Row, Col]Bool = false
    for b in cands:
        let p = simu(b, moves)
        if seen[p]: return false
        seen[p] = true
    true

fn query_filter(cands: List<Cell>, moves: Str) -> List<Cell>:
    let got: Cell = ask("?", moves, limit: 2)
    [b for b in cands if simu(b, moves) == got]

var cands = [(r, c) for r in Row for c in Col if (r, c) != (Row.first, Col.first)]

if h == 1:
    cands = query_filter(cands, repeat_str(">", w))
else:
    cands = query_filter(cands, sweep_moves())
    if cands.len > 1:
        var probe = fallback_probe()
        for r in 0..h:
            let m = row_probe(r)
            if separates(cands, m):
                probe = m
                break
        cands = query_filter(cands, probe)

print("!", cands[0].0, cands[0].1)

@interactor
fn judge(mut sol: Interactor):
    let h: 1..=100 = read()
    let w: 1..=100 = read()
    let br: 0..h = read()
    let bc: 0..w = read()
    assert h * w >= 2 and (br != 0 or bc != 0)
    sol.send(h, w)
    var queries = 0
    while true:
        let tag: Str = sol.recv()
        if tag == "?":
            let moves: Str = sol.recv()
            queries += 1
            assert queries <= 2 and moves.len <= 20_000
            var r = 0
            var c = 0
            for i in 0..moves.len:
                let ch = moves[i]
                var nr = r
                var nc = c
                if ch == '<': nc -= 1
                elif ch == '>': nc += 1
                elif ch == '^': nr -= 1
                elif ch == 'v': nr += 1
                else: assert false
                if nr in 0..h and nc in 0..w and (nr != br or nc != bc):
                    r = nr
                    c = nc
            sol.send(r, c)
        elif tag == "!":
            let (gr, gc): (Int, Int) = sol.recv()
            assert gr == br and gc == bc
            return
        else:
            assert false

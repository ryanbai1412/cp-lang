// EGOI 2023 P3, Find the Box — ideal CP Lang sketch
// Category: Interactive (stdio ?/!, tight 2-query budget, candidate-elimination simulator)
// Original C++: corpus/submissions/5678051.cpp (154 LOC raw, ~110 LOC stripped)
// Buildable today-form: proto/ports/scout_batch_19/today/egoi23p3.cp (verified vs orig C++)

let h: 1..=100 = read()
let w: 1..=100 = read()
assume h * w >= 2 // box cannot occupy (0, 0), so 1x1 is invalid

domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)

fn step(p: Cell, ch: Char) -> ?Cell:
    match ch:
        '<' => p.1 - 1 |> (c => (p.0, c))
        '>' => p.1 + 1 |> (c => (p.0, c))
        '^' => p.0 - 1 |> (r => (r, p.1))
        'v' => p.0 + 1 |> (r => (r, p.1))
        _   => none

fn simu(box: Cell, moves: Str) -> Cell:
    var pos: Cell = (Row.first, Col.first)
    for ch in moves:
        if let nxt = step(pos, ch) where nxt != box:
            pos = nxt
    pos

fn sweep_moves() -> Str:
    var out = ""
    for i in 1..h:
        if i % 2 == 1:
            out += "v>^" * (w - 1) + ">v>"
        else:
            out += "v<^" * (w - 1) + "<v<"
    out

fn row_probe(r: Int) -> Str = "v" * r + ">" * w

fn fallback_probe() -> Str =
    ">vv<^v" + ">" * w + "^v" + "<" * w + "^^" + ">" * w

fn separates(cands: List<Cell>, moves: Str) -> Bool:
    var seen: [Row, Col]Bool = false
    for b in cands:
        let p = simu(b, moves)
        if seen[p]: return false
        seen[p] = true
    true

fn query_filter(cands: List<Cell>, moves: Str) -> List<Cell>:
    // WISH B19-1: ask() with inline query limit + tuple destructuring
    let got: Cell = ask("?", moves, limit: 2)
    [b for b in cands if simu(b, moves) == got]

var cands = [(r, c) for r in Row for c in Col if (r, c) != (Row.first, Col.first)]

if h == 1:
    cands = query_filter(cands, ">" * w)
else:
    cands = query_filter(cands, sweep_moves())
    if cands.len > 1:
        let probe = (0..h).map(row_probe).find(m => separates(cands, m)) ?? fallback_probe()
        cands = query_filter(cands, probe)

print("!", cands[0].0, cands[0].1)

// WISH B19-2: co-located interactor with hidden @gen state, automatic shrinking, and query limits
interactor:
    // Hidden state generated alongside (h, w); `cpl stress` shrinks h, w, and box together
    hidden box: Cell = gen()
    assume box != (Row.first, Col.first)
    limit queries <= 2

    on ask("?", moves: Str) -> Cell:
        require moves.len <= 20000
        require all(ch in "<>^v" for ch in moves)
        simu(box, moves)

    check:
        let (tag, r, c): (Str, Row, Col) = read_output()
        require tag == "!"
        require (r, c) == box

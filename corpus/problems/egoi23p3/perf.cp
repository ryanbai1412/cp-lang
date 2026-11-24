// Tuned: the robot walks plain coordinates with one bounds test per move, as the accepted
// C++'s does, where the port steps through an optional cell built by `Row.at`/`Col.at`; the
// row probe is found by a loop, not `find`, whose optional holds a copy of the moves.
//
// Source: EGOI 2023 P3 (Find the Box), https://dmoj.ca/problem/egoi23p3
// limits: 1000 ms, 1024 MB
// Problem: interactive. An H x W grid (H, W <= 50) hides a solid box at some cell other than
//   (0, 0). Each query sends a move string (<= 20000 of <>^v) to a robot starting at (0, 0)
//   that stops at walls and the box, and reads where it ends. Find the box; full score needs
//   at most 2 queries.
// Solution: every other cell may hide the box; simulate each query against all of them and
//   keep those that predict the reported end. Query 1 snakes over the grid in v>^ steps (just
//   right to the wall when H = 1); if survivors remain, query 2 is the first row probe (down r
//   rows, then right to the wall) whose simulated ends tell them apart, else a fixed probe
//   through the top rows. O(H W |moves|) simulation per query.
// original: corpus/submissions/5678051.cpp
// Bounds: h, w: 1..=50
let wire = Wire()
let h: 1..=50 = read()
let w: 1..=50 = read()
domain Row = 0..h
domain Col = 0..w
type Cell = (Row, Col)
let start: Cell = (Row.first!, Col.first!)

fn robot(box: Cell, moves: Str) -> Cell:
    var (r, c) = (0, 0)
    let (br, bc) = (box.0.idx, box.1.idx)
    for move in moves:
        var (nr, nc) = (r, c)
        if move == '<': nc -= 1
        elif move == '>': nc += 1
        elif move == '^': nr -= 1
        else: nr += 1
        if 0 <= nr < h and 0 <= nc < w and not (nr == br and nc == bc): (r, c) = (nr, nc)
    (Row(r), Col(c))

fn tells_apart(boxes: List<Cell>, moves: Str) -> Bool =
    boxes.map(b => robot(b, moves)).distinct().len == boxes.len

fn ask(boxes: List<Cell>, moves: Str) -> List<Cell>:
    let got = wire.ask<(Row, Col)>("? {moves}")
    boxes.filter(b => robot(b, moves) == got).collect()

var snake = if h == 1: ">".repeat(w) else: ""
for r in 1..h:
    snake += if r % 2 == 1: "v>^".repeat(w - 1) + ">v>" else: "v<^".repeat(w - 1) + "<v<"
fn row_probe(r: Int) -> Str = "v".repeat(r) + ">".repeat(w)
let fallback = ">vv<^v" + ">".repeat(w) + "^v" + "<".repeat(w) + "^^" + ">".repeat(w)

let swept = ask([(r, c) for r in Row for c in Col if (r, c) != start], snake)
fn probe_for(boxes: List<Cell>) -> Str:
    for r in 0..h:
        if tells_apart(boxes, row_probe(r)): return row_probe(r)
    fallback
let found = if swept.len == 1: swept else: ask(swept, probe_for(swept))
print("!", found[0].0, found[0].1)

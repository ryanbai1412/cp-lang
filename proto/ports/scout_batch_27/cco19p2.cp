// CCO 2019 Sirtet: an n x m board of '#' and '.'. Each 4-connected group of
// '#' is a rigid piece, and gravity pulls every piece down until it rests
// on the floor or on a piece that rests. Print the final board.
//
// A piece falls as far as its tightest support lets it: the gap to the
// floor under any of its cells, or the gap to the next piece below in a
// column plus however far that piece falls. That is a shortest path from
// the floor, one edge per '#' cell.
// original: corpus/submissions/3613676.cpp
use std.dsu.{Dsu}
use std.graph.{dijkstra}

let (n, m): (Int, Int) = read()
domain Row = 0..n
domain Col = 0..m
domain Cell = 0..n * m + 1              // the cells, then the floor
let floor = Cell(n * m)
fn cell(r: Row, c: Col) -> Cell = Cell(r.to_idx * m + c.to_idx)

let board: [Row]Str = read()
fn full(r: Row, c: Col) -> Bool = board[r][c.to_idx] == '#'

var piece = Dsu<Cell>.new()
for r in Row:
    for c in Col:
        if not full(r, c): continue
        if let d = r.next:
            if full(d, c): piece.unite(cell(r, c), cell(d, c))
        if let d = c.next:
            if full(r, d): piece.unite(cell(r, c), cell(r, d))

// rests[p]: the pieces right above p in some column, with the gap between.
var rests: [Cell]List<(to: Cell, gap: Int)> = []
for c in Col:
    var (under, at) = (floor, n)        // the nearest piece below, and its row
    for r in Row.rev:
        if not full(r, c): continue
        let p = piece.find(cell(r, c))
        if p != under: rests[under].push((to: p, gap: at - r.to_idx - 1))
        under = p
        at = r.to_idx
let drop = dijkstra(p => rests[p], [(floor, 0)], a => a.gap)

var after: [Row]Str = [Row] _ => Str()
for r in Row:
    for c in Col: after[r].push('.')
for r in Row:
    for c in Col:
        if full(r, c): after[Row(r.to_idx + drop[piece.find(cell(r, c))]!)][c.to_idx] = '#'
for r in Row: print(after[r])

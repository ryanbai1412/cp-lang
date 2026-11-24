// Source: NOI 2023 Day 1 Square Coloring, https://dmoj.ca/problem/noi23p1
// limits: 1000 ms, 512 MB
// Problem: a grid of n columns and m rows (n, m <= 1e9), q <= 1e5 strokes painting the cells of a horizontal,
//   vertical or diagonal (x and y both rising) segment black; at most 5 strokes are diagonal.
//   Print the number of black cells. The first input number is the test id.
// Solution: compress each kind's line keys and give every line an IntervalSet, which merges its
//   overlapping or touching strokes; their covered lengths add up. A cell on a horizontal and a
//   vertical stroke is counted twice: sweep the rows upward with a Fenwick tree over the vertical
//   lines, live while one of their runs spans the row, and let each horizontal run take off the
//   live columns it spans. Each of the <= 5 diagonals meets every other line in at most one cell
//   per run: collect those columns, dedupe and take them off. O(q log q).
// original: corpus/submissions/5723056.cpp
// Bounds: _test_id: 0..=20; cols, rows: 1..=1e9; q: 1..=1e5; strokes: [q](1..=3, Col, Row, Col, Row)
use std.interval.{IntervalSet}

let _test_id: 0..=20 = read()
let cols: 1..=1e9 = read()
let rows: 1..=1e9 = read()
let q: 1..=1e5 = read()
domain Col = 1..=cols
domain Row = 1..=rows

let strokes: [q](1..=3, Col, Row, Col, Row) = read()

// one IntervalSet per line, over that line's compressed key, merges its overlapping or
// touching strokes
let (domain H, h_row) = compress([s.2 for s in strokes if s.0 == 1])
let (domain V, v_col) = compress([s.1 for s in strokes if s.0 == 2])
let (domain D, d_gap) = compress([s.2.ord - s.1.ord for s in strokes if s.0 == 3])
var hori = [H] _ => IntervalSet<Col>()
var vert = [V] _ => IntervalSet<Row>()
var diag = [D] _ => IntervalSet<Col>()  // the line y - x = d_gap
for (kind, x1, y1, x2, y2) in strokes:
    let across = min(x1, x2)..=max(x1, x2)
    match kind:
        1 => hori[h_row.index_of(y1)].add(across)
        2 => vert[v_col.index_of(x1)].add(min(y1, y2)..=max(y1, y2))
        _ => diag[d_gap.index_of(y1.ord - x1.ord)].add(across)

var cells = sum(line.covered for line in hori) + sum(line.covered for line in vert)
    + sum(line.covered for line in diag)

// rows upward: a vertical run goes live on its first row and dies after its last,
// and the horizontal runs of a row are asked in between
var live = Fenwick<V, Int>()
var events: List<(Row, Int, Col, Col)> = []  // (row, 0 enter / 1 ask / 2 leave, columns)
for v in V:
    for run in vert[v].each():
        events.push((run.first!, 0, v_col.get(v), v_col.get(v)))
        events.push((run.last!, 2, v_col.get(v), v_col.get(v)))
for h in H:
    for run in hori[h].each(): events.push((h_row.get(h), 1, run.first!, run.last!))
for (_, what, from, to) in events.sorted():
    if what == 1:
        if let lo = v_col.first_ge(from) and let hi = v_col.last_le(to) and lo <= hi:
            cells -= live.fold(lo..=hi)
    else: live.add(v_col.index_of(from), 1 - what)

// a diagonal run meets each other line in at most one cell
for d in D:
    let gap = d_gap.get(d)
    for run in diag[d].each():
        var hits: List<Col> = []
        for h in H:
            let x = h_row.get(h).ord - gap
            if x in 0..cols and Col.nth(x) in run and hori[h].contains(Col.nth(x)): hits.push(Col.nth(x))
        for v in V:
            let x = v_col.get(v)
            let y = x.ord + gap
            if x in run and y in 0..rows and vert[v].contains(Row.nth(y)): hits.push(x)
        hits.sort()
        hits.dedup()
        cells -= hits.len
print(cells)

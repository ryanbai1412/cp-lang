// IOI 2006 Pyramid: on a grid of heights, place a bw x bh base and, strictly
// inside it, a cw x ch chamber, so that the base minus the chamber is as
// high as possible in total. Print the two top-left corners (column, row).
// original: corpus/submissions/5034372.cpp
let w: 3..=1000 = read()
let h: 3..=1000 = read()
let bw: 3..=w = read()
let bh: 3..=h = read()
let cw: 1..=bw - 2 = read()
let ch: 1..=bh - 2 = read()
domain Row = 1..=h
domain Col = 1..=w
let height: [Row, Col]1..=100 = read()
let pre = prefix(height)

// A block is named by its rows and its columns; `windows` lists the placings.
type Block = (rows: Range<Row>, cols: Range<Col>)
let chambers = [Block] (rows, cols) in windows(Row, ch) * windows(Col, cw) => pre.sum(rows, cols)   // WISH F6, K14
// lowest chamber strictly inside each base, by a sliding minimum along rows and then columns
let lowest = chambers.sliding(min, rows: bh - ch - 1, cols: bw - cw - 1)                              // WISH K14

let base = argmax(windows(Row, bh) * windows(Col, bw), b => pre.sum(b.rows, b.cols) - lowest[b.inner.first])   // WISH K14
let room = argmin(windows(base.rows.inner, ch) * windows(base.cols.inner, cw), c => chambers[c])
print(base.cols.first, base.rows.first)
print(room.cols.first, room.rows.first)

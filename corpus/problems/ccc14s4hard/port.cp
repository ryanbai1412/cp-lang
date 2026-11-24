// Source: CCC 2014 S4 (Tinted Glass Window, hard data), https://dmoj.ca/problem/ccc14s4hard
// Problem: n axis-parallel panes, corners in 0..=1e9, each adding tint t_i; overlaps add up.
//   Print the total area whose tint is at least T. Too many panes for the O(n^2) grid of
//   ccc14s4.
// Solution: sweep x; between neighbouring y cuts the tint under the sweep line is constant, so
//   the line is a row of strips, a pane adds its tint to a range of strips at its left edge and
//   removes it at its right edge, and the area grows by dx times the height of the strips at or
//   over T. "Add to a range, measure what is over a threshold" has no segment tree, so the strips
//   are grouped in blocks of about sqrt(m): a block keeps its strips sorted by tint with running
//   heights and one pending add; a range add rebuilds at most two blocks and the measure is a
//   binary search per block. O(n sqrt(n) log n).
// original: corpus/submissions/5672273.cpp
let n: 1..=1e5 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()

let (domain Cut, ys) = compress([p.y0 for p in panes] + [p.y1 for p in panes])
domain Strip = 0..Cut.len - 1  // strip s lies between cuts s and s + 1
fn height(s: Strip) -> Int = ys.get(Cut.nth(s.ord + 1)) - ys.get(Cut.nth(s.ord))

let size = max(1, isqrt(Strip.len))
domain Block = 0..(Strip.len + size - 1) / size
fn strips(b: Block) -> Range<Strip> = Strip.nth(b.ord * size)..=Strip.nth(min(Strip.len, (b.ord + 1) * size) - 1)

var tint: [Strip]Int = 0
var pending: [Block]Int = 0
// darkest[b]: block b's strips, darkest first, each with the total height so far
var darkest: [Block]List<(tint: Int, height: Int)> = [Block] b => []

fn rebuild(b: Block):
    var rows: List<(tint: Int, height: Int)> = []
    var total = 0
    for s in strips(b).sorted(by: s => desc(tint[s])):
        total += height(s)
        rows.push((tint: tint[s], height: total))
    darkest[b] = rows

// The height of block b's strips at or over `need`.
fn dark(b: Block) -> Int:
    let lit = (0..darkest[b].len).first_true(i => darkest[b][i].tint + pending[b] < need) ?? darkest[b].len
    if lit == 0: 0 else: darkest[b][lit - 1].height

// Add t to the strips at offsets lo..hi.
fn add(lo: Int, hi: Int, t: Int):
    for b in Block:
        let (from, to) = (max(lo, strips(b).first!.ord), min(hi, strips(b).last!.ord + 1))
        if to - from == strips(b).len:
            pending[b] += t
        elif from < to:
            for s in from..to: tint[Strip.nth(s)] += t
            rebuild(b)

for b in Block: rebuild(b)
var events = [(x: p.x0, lo: ys.index_of(p.y0).ord, hi: ys.index_of(p.y1).ord, tint: p.tint) for p in panes] +
             [(x: p.x1, lo: ys.index_of(p.y0).ord, hi: ys.index_of(p.y1).ord, tint: -p.tint) for p in panes]
var (area, x) = (0, 0)
events.sort(by: e => e.x)
for e in events:
    area += (e.x - x) * sum(dark(b) for b in Block)
    x = e.x
    add(e.lo, e.hi, e.tint)
print(area)

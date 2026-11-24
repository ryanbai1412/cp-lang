// Source: CCC 2014 S4 (Tinted Glass Window, hard data), https://dmoj.ca/problem/ccc14s4hard
// Problem: n axis-parallel panes, corners in 0..=1e9, each adding tint t_i; overlaps add up.
//   Print the total area whose tint is at least T. Too many panes for the O(n^2) grid of
//   ccc14s4.
// Solution: sweep x; between neighbouring y cuts the tint under the sweep line is constant, so
//   the line is a row of strips, a pane adds its tint to a range of strips at its left edge and
//   removes it at its right edge, and the area grows by dx times the height of the strips at or
//   over T. "Add to a range, measure what is over a threshold" has no segment tree, so the strips
//   are grouped in blocks of about sqrt(m): a block keeps its strips sorted by tint with running
//   heights, one pending add and its lit height; a range add merges the touched strips back into
//   order in at most two blocks and re-measures each covered block by binary search.
//   O(n sqrt(n)) for the merges plus O(n sqrt(n) log n) comparisons for the measures.
// original: corpus/submissions/5672273.cpp
// Bounds: n: 1..=1e5; need, panes.tint: 1..=1e9;
//   panes.x0, panes.y0, panes.x1, panes.y1: 0..=1e9
let n: 1..=1e5 = read()
let need: 1..=1e9 = read()
let panes: [n](x0: 0..=1e9, y0: 0..=1e9, x1: 0..=1e9, y1: 0..=1e9, tint: 1..=1e9) = read()

let (domain Cut, ys) = compress([p.y0 for p in panes] + [p.y1 for p in panes])
domain Strip = 0..Cut.len - 1  // strip s lies between cuts s and s + 1
fn height(s: Strip) -> Int = ys.get(Cut.nth(s.ord + 1)) - ys.get(Cut.nth(s.ord))

let size = max(1, isqrt(Strip.len))
domain Block = 0..(Strip.len + size - 1) / size
fn strips(b: Block) -> Range<Strip> = Strip.nth(b.ord * size)..=Strip.nth(
    min(Strip.len, (b.ord + 1) * size) - 1
)

let tall: [Strip]Int = [Strip] s => height(s)
var pending: [Block]Int = 0
var rows: [Block]List<(strip: Strip, tint: Int)> = [Block] b => [
    (strip: s, tint: 0) for s in strips(b)
]
var upto: [Block]List<Int> = [Block] b => [0 for _ in strips(b)]  // running heights along rows, darkest first
var shade: [Block]Int = 0                                         // height at or over need
var shaded = 0

fn heights(b: Block):
    var total = 0
    for (i, r) in rows[b].items:
        total += tall[r.strip]
        upto[b][i] = total

fn measure(b: Block):
    let lit = (0..rows[b].len).first_true(i => rows[b][i].tint + pending[b] < need) ?? rows[b].len
    shaded -= shade[b]
    shade[b] = if lit == 0: 0 else: upto[b][lit - 1]
    shaded += shade[b]

fn touch(b: Block, lo: Int, hi: Int, t: Int):
    let (from, to) = (max(lo, strips(b).first!.ord), min(hi, strips(b).last!.ord + 1))
    if to - from == strips(b).len:
        pending[b] += t
    else:
        let (hit, rest) = rows[b].partition(r => from <= r.strip.ord and r.strip.ord < to)
        let lifted = [(strip: r.strip, tint: r.tint + t) for r in hit]
        rows[b] = lifted.merge_sorted(rest, by: r => -r.tint).collect()
        heights(b)
    measure(b)

fn add(lo: Int, hi: Int, t: Int):
    let (first, last) = (Block.nth(lo / size), Block.nth((hi - 1) / size))
    touch(first, lo, hi, t)
    if first == last: return
    for b in first.next!..last:
        pending[b] += t
        measure(b)
    touch(last, lo, hi, t)

for b in Block: heights(b)
var events = [
    (x: p.x0, lo: ys.index_of(p.y0).ord, hi: ys.index_of(p.y1).ord, tint: p.tint) for p in panes
] + [
    (x: p.x1, lo: ys.index_of(p.y0).ord, hi: ys.index_of(p.y1).ord, tint: -p.tint) for p in panes
]
var (area, x) = (0, 0)
events.sort(by: e => e.x)
for e in events:
    area += (e.x - x) * shaded
    x = e.x
    if e.lo < e.hi: add(e.lo, e.hi, e.tint)
print(area)

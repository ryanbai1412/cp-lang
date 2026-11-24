// Source: CCC 2014 S4 (Tinted Glass Window, large limits), https://dmoj.ca/problem/ccc14s4hard
// Problem: n rectangular panes, each with a tint, lie over a window. Print the area
//   where the tints of the panes on top of each other add up to at least `need`.
// Solution: sweep a vertical line; between neighbouring y coordinates the line is a row
//   of strips, and a pane adds its tint to a range of strips at its left edge and takes
//   it back at its right edge. Strips are grouped in blocks of about sqrt(m), each
//   sorted by tint with running heights and one pending add: a range add re-sorts at
//   most two blocks, and the height at or over `need` is a binary search per block,
//   times the distance to the next event.
let (n, need): (Int, Int) = read()
domain Pane = 0..n
let panes: [Pane](x1: Int, y1: Int, x2: Int, y2: Int, tint: Int) = read()

let ys = Set([panes[p].y1 for p in Pane] + [panes[p].y2 for p in Pane]).sorted()
domain Strip = 0..ys.len - 1  // strip s lies between ys[s] and ys[s + 1]
var first_strip = Map<Int, Int>()
for i in 0..ys.len: first_strip[ys[i]] = i

let size = max(1, isqrt(Strip.len))
domain Block = 0..(Strip.len + size - 1) / size
fn strips(b: Block) -> Range<Strip> = Strip.nth(b.ord * size)..=Strip.nth(
    min(Strip.len, (b.ord + 1) * size) - 1
)

var tint: [Strip]Int = 0
var pending: [Block]Int = 0
var darkest: [Block]List<(tint: Int, height: Int)> = [Block] b => []  // darkest first; height is the total so far

fn rebuild(b: Block):
    var (rows, height) = ([], 0)
    for s in strips(b).sorted(by: s => -tint[s]):
        height += ys[s.ord + 1] - ys[s.ord]
        rows.push((tint: tint[s], height:))
    darkest[b] = rows

// The height of the strips of block b that are at or over `need`.
fn tinted(b: Block) -> Int:
    let rows = darkest[b]
    let lo = (0..rows.len).first_true(mid => rows[mid].tint + pending[b] < need) ?? rows.len  // rows[..lo] are dark enough, rows[hi..] are not
    if lo == 0: 0 else: rows[lo - 1].height

// Add t to the strips at offsets l..r.
fn add(l: Int, r: Int, t: Int):
    for b in Block:
        let (from, to) = (max(l, strips(b).first!.ord), min(r, strips(b).last!.ord + 1))
        if to - from == strips(b).len:
            pending[b] += t
        elif from < to:
            for s in from..to: tint[Strip.nth(s)] += t
            rebuild(b)

for b in Block: rebuild(b)
var events: List<(x: Int, l: Int, r: Int, tint: Int)> = []
for p in Pane:
    let (x1, y1, x2, y2, t) = panes[p]
    events.push((x: x1, l: first_strip[y1], r: first_strip[y2], tint: t))
    events.push((x: x2, l: first_strip[y1], r: first_strip[y2], tint: -t))

var (area, x) = (0, 0)
for e in events.sorted(by: e => e.x):
    if e.x > x: area += (e.x - x) * sum(tinted(b) for b in Block)
    x = e.x
    add(e.l, e.r, e.tint)
print(area)

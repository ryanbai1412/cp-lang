// Source: NOI 2005 P1 (Gorgeous Waltz), https://dmoj.ca/problem/noi05p1
// limits: 400 ms, 256 MB
// Problem: an N x M grid (N, M <= 200) with furniture 'x' and a piano at (x, y). Times 1..=T
//   (T <= 40000) form K <= 200 intervals, each tilting the ship N, S, W or E; each second the
//   piano slides one cell that way or is held, never into furniture or off the grid. Print
//   the longest total slide.
// Solution: DP per interval, not per second: far[p] is the longest slide ending at p. In an
//   interval of len seconds the piano moves 0..=len cells along its line without crossing
//   furniture, so next[p] = max(far[q] + dist(q, p)) over q at most len behind p. Walk each
//   row or column in the tilt direction with a monotone deque of (far - place), reset at
//   furniture. O(K N M).
// original: corpus/submissions/5032738.cpp
// Bounds: n, m, q: 1..=200; tilts.way: 1..=4; start.0: 1..=n; start.1: 1..=m
let (n, m): (1..=200, 1..=200) = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let start: Cell = read()
let q: 1..=200 = read()
let hall: [Row, Col]Char = read()
domain Tilt = 1..=q
let tilts: [Tilt](from: Int, to: Int, way: 1..=4) = read()

// the first cell of every row or column, walked the way the ship tilts (1 N, 2 S, 3 W, 4 E)
fn starts(way: 1..=4) -> yields Cell:
    if way <= 2:
        let r = if way == 1: Row.last! else: Row.first!
        for c in Col: yield (r, c)
    else:
        let c = if way == 3: Col.last! else: Col.first!
        for r in Row: yield (r, c)

fn along(way: 1..=4) -> Dir4 = match way:
    1 => Dir4.up
    2 => Dir4.down
    3 => Dir4.left
    _ => Dir4.right

fn line(s: Cell, d: Dir4) -> yields Cell:
    yield s
    for p in s.ray(d): yield p

var far: [Row, Col]?Int = None  // the longest slide that ends here
far[start] = 0
for (from, to, way) in tilts:
    let seconds = to - from + 1
    var next: [Row, Col]?Int = None
    for s in starts(way):
        var best = Window(max)  // the best slide - place of the starts still in reach, keyed by place
        var i = -1
        for p in line(s, along(way)):
            i += 1
            if hall[p] == 'x':
                best = Window(max)
                continue
            if let slid = far[p]: best.push(slid - i, at: i)
            best.expire(before: i - seconds)
            if let v = best.all: next[p] = v + i
    far = next
print(max(far[r, c] ?? 0 for r in Row for c in Col)!)

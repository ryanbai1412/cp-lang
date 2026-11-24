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
// Bounds: n, m, q: 1..=200; tilts.way: 1..=4
let n: 1..=200 = read()
let m: 1..=200 = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let start: Cell = read()
let q: 1..=200 = read()
let hall: [Row, Col]Char = read()
domain Tilt = 1..=q
let tilts: [Tilt](from: Int, to: Int, way: 1..=4) = read()  // WISH L2: way: Dir4, numbered N S W E

// Every row or column of the hall, walked in the direction of the tilt.
fn lines(way: Int) -> List<List<Cell>>:  // WISH L3: for line in Cell.lines(way)
    if way == 1: return [[(r, c) for r in Row.rev] for c in Col]
    if way == 2: return [[(r, c) for r in Row] for c in Col]
    if way == 3: return [[(r, c) for c in Col.rev] for r in Row]
    return [[(r, c) for c in Col] for r in Row]

var far: [Row, Col]?Int = None  // longest slide that ends here
far[start] = 0
for (from, to, way) in tilts:
    let seconds = to - from + 1
    var next: [Row, Col]?Int = None
    for line in lines(way):
        // (slide minus place, place) of the starts still in reach, best first
        var reach: List<(Int, Int)> = []  // WISH L15: sliding(max, xs, width) over a MonoQueue
        var head = 0
        for i in 0..line.len:
            let p = line[i]
            if hall[p] == 'x':  // WISH L16: for run in line.split(p => hall[p] == 'x')
                reach = []
                head = 0
            else:
                if let slid = far[p]:
                    while reach.len > head and reach.last!.0 <= slid - i: reach.pop()
                    reach.push((slid - i, i))
                if head < reach.len and reach[head].1 < i - seconds: head += 1
                if head < reach.len: next[p] = reach[head].0 + i
    far = next
print([far[r, c] ?? 0 for r in Row for c in Col].max()!)

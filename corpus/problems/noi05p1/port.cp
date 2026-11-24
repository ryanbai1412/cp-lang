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
let (n, m): (1..=200, 1..=200) = read()
domain Row = 1..=n
domain Col = 1..=m
type Cell = (Row, Col)
let start: Cell = read()
let q: 1..=200 = read()
let hall: [Row, Col]Char = read()
domain Tilt = 1..=q
let tilts: [Tilt](from: Int, to: Int, way: 1..=4) = read()

// every row or column of the hall, walked in the tilt's direction (1 N, 2 S, 3 W, 4 E)
fn lines(way: 1..=4) -> List<List<Cell>> = match way:
    1 => [[(r, c) for r in Row.rev] for c in Col]
    2 => [[(r, c) for r in Row] for c in Col]
    3 => [[(r, c) for c in Col.rev] for r in Row]
    _ => [[(r, c) for c in Col] for r in Row]

var far: [Row, Col]?Int = None  // the longest slide that ends here
far[start] = 0
for (from, to, way) in tilts:
    let seconds = to - from + 1
    var next: [Row, Col]?Int = None
    for line in lines(way):
        var best = Window(max)  // the best slide - place of the starts still in reach
        var reach: Deque<(Int, Int)> = Deque.new()  // (slide - place, place), oldest first
        for (i, p) in line.items:
            if hall[p] == 'x':
                (best, reach) = (Window(max), Deque.new())
                continue
            if let slid = far[p]:
                best.push(slid - i)
                reach.push((slid - i, i))
            for o in reach.pop_front_while(o => o.1 < i - seconds): best.pop(o.0)
            if let v = best.all: next[p] = v + i
    far = next
print(max(far[r, c] ?? 0 for r in Row for c in Col)!)

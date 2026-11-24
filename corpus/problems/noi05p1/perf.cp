// Tuned: slides live in plain Int grids (-1 where the piano can't be); a generator walks each
// direction's lines as (cell, place), and each line keeps one monotone deque of
// (slide - place, place) in two plain lists, as the accepted C++ does, instead of a Window
// beside a Deque.
//
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
// every row or column of the hall, walked in a tilt's direction (1 N, 2 S, 3 W, 4 E), as
// (cell, place on its line)
fn lines_of(way: Int) -> yields (r: Row, c: Col, i: Int):
    match way:
        1 => for c in Col: for r in Row.rev: yield (r:, c:, i: n - r.label)
        2 => for c in Col: for r in Row: yield (r:, c:, i: r.label - 1)
        3 => for r in Row: for c in Col.rev: yield (r:, c:, i: m - c.label)
        _ => for r in Row: for c in Col: yield (r:, c:, i: c.label - 1)
var far: [Row, Col]Int = -1  // the longest slide that ends here, -1 if none
far[start] = 0
var next: [Row, Col]Int = -1
let long = max(n, m)
var (gain, place) = (List.filled(long, 0), List.filled(long, 0))  // the deque, oldest first
for (from, to, way) in tilts:
    let seconds = to - from + 1
    var (head, tail) = (0, 0)
    for (r, c, i) in lines_of(way):
        if i == 0 or hall[r, c] == 'x': (head, tail) = (0, 0)
        if hall[r, c] == 'x':
            next[r, c] = -1
            continue
        let slid = far[r, c]
        if slid >= 0:
            while tail > head and gain[tail - 1] <= slid - i: tail -= 1
            (gain[tail], place[tail]) = (slid - i, i)
            tail += 1
        while tail > head and place[head] < i - seconds: head += 1
        next[r, c] = if tail > head: gain[head] + i else: -1
    (far, next) = (next, far)
print(max(far[r, c] for r in Row for c in Col)!)

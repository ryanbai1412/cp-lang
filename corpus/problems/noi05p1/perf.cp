// Tuned: slides live in flat Int lists (-1 where the piano can't be); each tilt walks its
// direction's lines by stride, and each line keeps one monotone deque of
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
// the hall row by row as one flat list: cell (r, c) is at (r - 1) * m + c - 1
let wall = [hall[r, c] == 'x' for r in Row for c in Col]
var far = List.filled(n * m, -1)  // the longest slide that ends here, -1 if none
far[(start.0.label - 1) * m + start.1.label - 1] = 0
var next = List.filled(n * m, -1)
let long = max(n, m)
var (gain, place) = (List.filled(long, 0), List.filled(long, 0))  // the deque, oldest first
for (from, to, way) in tilts:
    let seconds = to - from + 1
    // every row or column of the hall, walked in the tilt's direction (1 N, 2 S, 3 W, 4 E):
    // line j starts at first + j * across and steps by along
    let (lines, len, first, across, along) = match way:
        1 => (m, n, (n - 1) * m, 1, -m)
        2 => (m, n, 0, 1, m)
        3 => (n, m, m - 1, m, -1)
        _ => (n, m, 0, m, 1)
    for j in 0..lines:
        var (head, tail) = (0, 0)
        var at = first + j * across
        for i in 0..len:
            if wall[at]:
                (head, tail) = (0, 0)
                next[at] = -1
            else:
                let slid = far[at]
                if slid >= 0:
                    while tail > head and gain[tail - 1] <= slid - i: tail -= 1
                    (gain[tail], place[tail]) = (slid - i, i)
                    tail += 1
                while tail > head and place[head] < i - seconds: head += 1
                next[at] = if tail > head: gain[head] + i else: -1
            at += along
    swap(mut far, mut next)
print(far.max()!)

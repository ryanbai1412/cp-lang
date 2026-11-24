// Source: Yet Another Contest 7 P2 (Coprime Grid), https://dmoj.ca/problem/yac7p2
// limits: 2000 ms, 256 MB
// Problem: fill an n x m grid with 1..=n*m, each once, so that every cell is coprime with at
//   least two side-adjacent cells; print the grid, or -1 if impossible. T, n, m <= 100.
// Solution: consecutive integers are coprime, so number the cells along a Hamiltonian path whose
//   two ends also touch a coprime cell; a 1 x m grid fails at its ends (one neighbour). With an
//   even side (made the height, transposing if needed) row 0, a snake over columns 1.. and
//   column 0 back up close a cycle, so n*m sits next to 1. With both sides odd the path runs
//   along row 0, down the last column, snakes the columns down to 2, then zigzags columns 0-1
//   up to (1, 1), which is next to 2 at (0, 1) while n*m is odd. O(nm) per test.
// original: corpus/submissions/5705152.cpp
// Bounds: t, n, m: 1..=100

// the cells of an h x w grid (h even, or both odd), w >= 2, in path order from (0, 0)
fn tour(h: Int, w: Int) -> List<(Int, Int)>:
    let top = [(0, c) for c in 0..w]
    if h % 2 == 0:
        let snake = [(r, if r % 2 == 1: w - k else: k) for r in 1..h for k in 1..w]
        return List(chain(top, snake, [(r, 0) for r in (1..h).rev]))
    let side = [(r, w - 1) for r in 1..h]
    let snake = [(if (w - c) % 2 == 0: h - k else: k, c) for c in (2..w - 1).rev for k in 1..h]
    let zigzag = [(r, if r % 2 == 0: 1 - k else: k) for r in (1..h).rev for k in 0..2]
    List(chain(top, side, snake, zigzag))

let t: 1..=100 = read()
for _ in 0..t:
    let (n, m): (1..=100, 1..=100) = read()
    if n == 1 or m == 1:
        print(-1)
        continue
    // an even side must be the height
    let flip = n % 2 == 1 and m % 2 == 0
    let (h, w) = if flip: (m, n) else: (n, m)
    var number: [0..h, 0..w]Int = 0
    for (i, (r, c)) in tour(h, w).items: number[r, c] = i + 1
    for i in 0..n: print([if flip: number[j, i] else: number[i, j] for j in 0..m])

// Source: Yet Another Contest 7 P2 (Coprime Grid), https://dmoj.ca/problem/yac7p2
// limits: 2000 ms, 256 MB
// Problem: fill an n x m grid with 1..=n*m, each once, so that every cell is coprime with at
//   least two side-adjacent cells; print the grid, or -1 if impossible. T, n, m <= 100.
// Solution: consecutive integers are coprime, so number the cells along a Hamiltonian path whose
//   two ends also touch a coprime cell; a 1 x m grid fails at its ends (one neighbour). With an
//   even side (made the height, transposing if needed) a snake closes into a cycle back next to
//   cell 1. With both sides odd, an open path ends at (1, 1), next to 1 and next to 2 (n*m is
//   odd). O(nm) per test.
// original: corpus/submissions/5705152.cpp
// Bounds: t: 1..=100_000

let t: 1..=100_000 = read()

var cell: List<Int> = []
var w = 0
var cx = 0
var cy = 0
var ind = 0

// walk cnt cells in direction (dx, dy), numbering each one
fn go(dx: Int, dy: Int, cnt: Int):
    for _ in 0..cnt:
        cx += dx
        cy += dy
        ind += 1
        cell[cx * w + cy] = ind

fn right(c: Int):
    go(0, 1, c)
fn left(c: Int):
    go(0, -1, c)
fn down(c: Int):
    go(1, 0, c)
fn up(c: Int):
    go(-1, 0, c)

// h even: snake through columns 1.., come home along column 0
fn tour_even(h: Int):
    right(w - 1)
    var i = 3
    while i < h:
        down(1)
        left(w - 2)
        down(1)
        right(w - 2)
        i += 2
    down(1)
    left(w - 1)
    up(h - 2)

// h and w odd: the original's open path
fn tour_odd(h: Int):
    right(w - 1)
    down(h - 1)
    var i = w - 3
    while i > 1:
        left(1)
        up(h - 2)
        left(1)
        down(h - 2)
        i -= 2
    left(1)
    i = h - 2
    while i >= 3:
        left(1)
        up(1)
        right(1)
        up(1)
        i -= 2
    left(1)
    up(1)
    right(1)

for _ in 0..t:
    let n: 1..=1000 = read()
    let m: 1..=1000 = read()
    if n == 1 or m == 1:
        print(-1)
    else:
        // an even side must be the height; transpose when only m is even
        let flip = n % 2 == 1 and m % 2 == 0
        var h = n
        w = m
        if flip:
            h = m
            w = n
        cell.clear()
        for _ in 0..h * w:
            cell.push(0)
        cell[0] = 1
        cx = 0
        cy = 0
        ind = 1
        if h % 2 == 0:
            tour_even(h)
        else:
            tour_odd(h)
        for i in 0..n:
            var row: List<Int> = []
            for j in 0..m:
                if flip:
                    row.push(cell[j * w + i])
                else:
                    row.push(cell[i * w + j])
            print(row)

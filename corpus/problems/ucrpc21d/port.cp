// Source: UCRPC Fall 2021 D, Stake Your Claim, https://dmoj.ca/problem/ucrpc21d
// limits: 1000 ms, 256 MB
// Problem: an r x c board (0 <= r, c <= 1000) drawn as 2r+1 lines of 2c+1 characters: cells at
//   even 1-based rows and columns, walls '+', '-', '|', open '.' elsewhere. The walls cut the
//   board into four regions; print their areas (cell counts) in decreasing order; four regions imply rc >= 4.
// Solution: the rooms are an r x c grid; join each room to its right and lower neighbour when
//   the canvas character between them is open, in a std Dsu over rooms, and print the four
//   region sizes from largest down. O(rc α).
// original: corpus/submissions/3944584.cpp
// Bounds: r, c: 0..=1000
let (r, c): (0..=1000, 0..=1000) = read()
domain Row = 0..r
domain Col = 0..c
domain Room = (Row, Col)
let board: [0..=2 * r, 0..=2 * c]Char = read()

fn open(y: Int, x: Int) -> Bool: board[y, x] == '.'

var regions = Dsu<Room>()
for i in Row:
    for j in Col:
        if let k = j.next, open(2 * Int(i) + 1, 2 * Int(k)): regions.unite((i, j), (i, k))
        if let k = i.next, open(2 * Int(k), 2 * Int(j) + 1): regions.unite((i, j), (k, j))
let areas = [regions.size(room) for room in regions.roots()].sorted(by: a => desc(a))
print(areas[..4])

// Source: CCC 2023 S1 (Trianglane), https://dmoj.ca/problem/ccc23s1
// Problem: two rows of c <= 2e5 unit triangle tiles (the first points up, neighbours alternate),
//   each black (1) or white (0). Print the length of tape around the black area: the black
//   tiles' sides not shared with another black tile.
// Solution: each black tile adds 3, minus 1 for each black tile it shares a side with: its left
//   and right neighbours, and at even columns the tile in the other row. O(c).
// original: corpus/submissions/5350804.cpp

fn main_solve():
    let n: Int = read()
    domain Row = 0..2
    domain Col = 0..n
    let wet: [Row, Col]Int = read()
    var ans = 0
    for r in 0..2:
        for c in 0..n:
            if wet[Row(r), Col(c)] == 0: continue
            ans += 3
            if c > 0 and wet[Row(r), Col(c - 1)] == 1: ans -= 1
            if c + 1 < n and wet[Row(r), Col(c + 1)] == 1: ans -= 1
            if c % 2 == 0 and wet[Row(1 - r), Col(c)] == 1: ans -= 1
    print(ans)

main_solve()

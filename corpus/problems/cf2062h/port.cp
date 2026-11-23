// Source: Codeforces 2062H (Galaxy Generator), https://codeforces.com/contest/2062/problem/H
// Problem: an n x n 0/1 star matrix, n <= 14 (sum of 2^n <= 2^14, multitest). A set's value is the
//   fewest galaxies (connected components, joined along shared rows or columns) left after
//   repeatedly adding a star at a point that would connect to >= 3 stars. Sum the values of all
//   non-empty subsets of the stars, mod 1e9+7.
// Solution: merging stops exactly when the galaxies' bounding boxes have pairwise disjoint row and
//   column projections, so a set's value is its number of boxes. ways[hi][lo][cols] counts star
//   sets of rows lo..<hi whose boxes all lie in the columns cols; galaxies[..] sums their boxes.
//   A box exactly rows mid..<hi x band takes 2^(stars inside) minus the sets already counted as
//   smaller or several boxes, and is stacked on the sets below mid in the other columns.
//   O(2^n n^4).
type Z = Mod<1_000_000_007>
@!multitest
let n: 1..=14 = read()
domain Row = 0..n
domain Col = 0..n
let grid: [Row]Str = read()
var stars: [Row]Subset<Col> = Subset<Col>.empty
for r in Row:
    for c in Col:
        if grid[r][c.to_idx] == '1': stars[r] = stars[r].with(c)
var ways: [0..=n][0..=n][Subset<Col>]Z = 0
var galaxies: [0..=n][0..=n][Subset<Col>]Z = 0
for hi in 0..=n:
    ways[hi][hi].fill(1)
    for mid in (0..hi).rev:
        for cols in Subset<Col>:
            ways[hi][mid][cols] += ways[hi - 1][mid][cols]
            galaxies[hi][mid][cols] += galaxies[hi - 1][mid][cols]
        for y_lo in (0..n).rev:
            var band = Subset<Col>.empty
            for y in y_lo..n:
                band = band.with(Col(y))
                let inside = sum((stars[Row(r)] & band).len for r in mid..hi)
                let one_box = Z(2) ** inside - ways[hi][mid][band]
                for lo in 0..=mid:
                    for rest in (Subset<Col>.full - band).subsets:
                        let (w, g) = (ways[mid][lo][rest], galaxies[mid][lo][rest])
                        ways[hi][lo][rest | band] += w * one_box
                        galaxies[hi][lo][rest | band] += (w + g) * one_box
print(galaxies[n][0][Subset<Col>.full])

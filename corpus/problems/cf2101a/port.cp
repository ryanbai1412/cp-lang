// Source: Codeforces 2101A (Mex in the Grid), https://codeforces.com/contest/2101/problem/A
// limits: 2000 ms, 250 MB
// Problem: multitest; n <= 500 (sum of n <= 1000). Place 0..n^2 - 1 in an n x n grid to
//   maximise the sum of mex over all subgrids; print any optimal grid.
// Solution: a subgrid has mex >= k iff it covers the bounding box of 0..k - 1, and boxes
//   covered by the most subgrids are small and central. Growing the values in square shells
//   around the centre keeps every prefix 0..k - 1 in the smallest central box, so every term
//   is maximal. rank orders rows (and columns) outward from the centre, alternating sides;
//   shell k (cells with larger rank k) gets k^2.. : its row k first, then its column k. O(n^2).
@!multitest
let n: 1..=500 = read()
domain Coord = 0..n
let rank: [Coord]Int = [Coord] x => if x.ord >= n / 2: 2 * (x.ord - n / 2) else: 2 * (n / 2 - x.ord) - 1
let grid: [Coord, Coord]Int = [Coord, Coord] (r, c) =>
    let k = max(rank[r], rank[c])
    k * k + (if rank[c] < k: rank[c] else: k + rank[r])
for r in Coord: print(grid[r, ..])

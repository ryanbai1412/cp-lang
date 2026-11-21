// Codeforces 2101 A: Mex in the Grid (dream).
// A subgrid has mex >= k iff it holds every cell of 0..k - 1, i.e. it covers
// their bounding box, and the number of subgrids covering a box is largest
// when the box is small and central. Growing values in square shells around
// the centre keeps every prefix 0..k - 1 in the smallest central box at once,
// so every term of the sum is maximal.
//
// `rank` orders rows (and columns) outward from the centre, alternating
// sides. Shell k is the cells whose larger rank is k: its row k is filled
// first, then its column k, so the k * k smaller shells come before it.
@!multitest
let n: 1..=500 = read()
domain Coord = 0..n
let rank: [Coord]Int = [Coord] x => if x.ord >= n / 2: 2 * (x.ord - n / 2) else: 2 * (n / 2 - x.ord) - 1
let grid: [Coord, Coord]Int = [Coord, Coord] (r, c) =>
    let k = max(rank[r], rank[c])
    k * k + (if rank[c] < k: rank[c] else: k + rank[r])
for r in Coord: print(grid[r, ..])

// CCC 2019 S5 Triangle: The Data Structure. A triangle of n rows, row r
// holding r numbers. Print the sum, over every sub-triangle of size k, of
// its largest number.
let n: 1..=3000 = read()
let k: 1..=n = read()
domain Row = 1..=n
type Cell = (r: Row, c: Row) where c <= r                           // WISH F4: a triangular index
let tri: [Cell]0..=1e9 = read()

// best[r, c]: the largest number in the triangle of the current size whose
// apex is (r, c). Three triangles of one size, pushed apart by `step`, cover
// one triangle up to half again as large, so the size reaches k in O(log k)
// passes. Rows are updated top down, so a pass reads only the previous size.
var best = tri
var size = 1
while size < k:
    let step = min(k, (3 * size + 1) / 2) - size
    for (r, c) in Cell:
        if let below = r + step and let right = c + step:
            best[r, c] = max(best[r, c], best[below, c], best[below, right])
    size += step
print(sum(best[r, c] for (r, c) in Cell if r.ord + k <= n))

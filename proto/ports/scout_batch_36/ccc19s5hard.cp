// CCC '19 S5 (Hard) — Triangle: The Data Structure
// original: corpus/submissions/5119931.cpp
//
// Maintain the maximum over every size-s subtriangle in a flat triangular
// array (row i, col j -> i * (i + 1) / 2 + j), growing s by 3/2 each pass
// until s == k so each cell is covered by 3 overlapping size-pn subtriangles.

let n: 1..=3000 = read()
let k: 1..=n = read()

fn tri(i: Int, j: Int) -> Int: i * (i + 1) / 2 + j

let total = n * (n + 1) / 2
var cur: List<Int> = [0; total]
for i in 0..n:
    for j in 0..=i:
        cur[tri(i, j)] = read()

var nxt: List<Int> = [0; total]
var side = 1
while side < k:
    let next_side = min(k, (side * 3 + 1) / 2)
    let d = next_side - side
    for i in 0..=n - next_side:
        for j in 0..=i:
            nxt[tri(i, j)] = max(
                cur[tri(i, j)],
                cur[tri(i + d, j)],
                cur[tri(i + d, j + d)],
            )
    (cur, nxt) = (nxt, cur)
    side = next_side

var ans = 0
for i in 0..=n - k:
    for j in 0..=i:
        ans += cur[tri(i, j)]
print(ans)

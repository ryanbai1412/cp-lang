// Matched benchmark: 200x200 max-plus Mat<?Int> ** 10^6, a third of the entries None.

domain V = 0..200
var s = 42
var grid: [V, V]?Int = [V, V] (_, _) => None
unchecked:
    for i in V:
        for j in V:
            s = (s * 1103515245 + 12345) & 2_147_483_647
            if s % 3 != 0: grid[i, j] = Some(s % 1000)

let r = Mat(grid, over: Semiring(max, +)) ** 1_000_000
var acc = 0
var gaps = 0
unchecked:
    for i in 0..r.rows:
        for j in 0..r.cols:
            if let v = r.get(i, j): acc = (
                acc + v % 1_000_003 * (i * r.cols + j + 1)
            ) % 1_000_000_007
            else: gaps += 1
print(acc, gaps)

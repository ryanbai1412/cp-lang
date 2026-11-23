// CF 1807F Bouncy Ball
// in: t; per test n m i1 j1 i2 j2 d (d in DR DL UR UL). out: bounces before first reaching (i2, j2), or -1.
@!multitest
let n, m: 2..=25_000 = read()
var i: 1..=n = read()
var j: 1..=m = read()
let gi: 1..=n = read()
let gj: 1..=m = read()
var (di, dj) = match read<"DR" | "DL" | "UR" | "UL">():
    "DR" => (1, 1)
    "DL" => (1, -1)
    "UR" => (-1, 1)
    "UL" => (-1, -1)

var bounces = 0
// 4nm states (cell, direction): the walk repeats after that
repeat 4 * n * m:
    if i == gi and j == gj:
        print(bounces)
        return
    let flip_i = not (1 <= i + di <= n)
    let flip_j = not (1 <= j + dj <= m)
    if flip_i: di = -di
    if flip_j: dj = -dj
    if flip_i or flip_j: bounces += 1
    i += di
    j += dj
print(-1)

use std.math.{splitmix64}

@!multitest
let (n, q): (Int, Int) = read()
var (row0, freq0) = ([0 for _ in 0..n], [0 for _ in 0..n])
var (row1, freq1) = ([0 for _ in 0..n], [0 for _ in 0..n])
var (h0, h1): (U64, U64) = (0, 0)
for _ in 0..q:
    let (r0, r1): (Int, Int) = read()
    let f0 = row0[r0 - 1]
    row0[r0 - 1] = f0 + 1
    let g0 = freq0[f0]
    freq0[f0] = g0 + 1
    h0 += splitmix64((U64(f0) << 32) | U64(g0))
    let f1 = row1[r1 - 1]
    row1[r1 - 1] = f1 + 1
    let g1 = freq1[f1]
    freq1[f1] = g1 + 1
    h1 += splitmix64((U64(g1) << 32) | U64(f1))
    print(if h0 == h1: "YES" else: "NO")

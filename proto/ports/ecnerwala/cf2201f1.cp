// Codeforces 2201 F1: check whether the transposed Ferrers diagrams of the two
// row-frequency multisets match after each update via a splitmix64 multiset hash.
use std.math.{splitmix64}

@!multitest
let (n, q): (Int, Int) = read()
var row_freq: [0..2, 0..n]Int = 0
var freq_freq: [0..2, 0..n]Int = 0
var h: [0..2]U64 = 0

for _ in 0..q:
    let rs: [0..2]Int = read()
    for z in 0..2:
        let f = row_freq[z, rs[z] - 1]
        row_freq[z, rs[z] - 1] = f + 1
        let g = freq_freq[z, f]
        freq_freq[z, f] = g + 1
        let (a, b) = if z == 0: (f, g) else: (g, f)
        h[z] += splitmix64((U64(a) << 32) | U64(b))
    print(if h[0] == h[1]: "YES" else: "NO")

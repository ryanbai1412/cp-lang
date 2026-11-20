// SAC '22 CC5 P5 (today/ runnable subset) — original: corpus/submissions/4575631.cpp

const MOD1 = 1_000_000_007
const MOD2 = 1_000_000_009
const BR1 = 131
const BR2 = 137
const BC1 = 100_003
const BC2 = 100_019

let n: 1..=5_000 = read()
let m: 1..=5_000 = read()
let k: 1..=5_000 = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()

var pr1: [0..=m]Int = 1
var pr2: [0..=m]Int = 1
for i in 1..=m:
    pr1[i] = (pr1[i - 1] * BR1) % MOD1
    pr2[i] = (pr2[i - 1] * BR2) % MOD2

var pc1: [0..=n]Int = 1
var pc2: [0..=n]Int = 1
for i in 1..=n:
    pc1[i] = (pc1[i - 1] * BC1) % MOD1
    pc2[i] = (pc2[i - 1] * BC2) % MOD2

var rh1: [0..n, 0..=m]Int = 0
var rh2: [0..n, 0..=m]Int = 0
for r in 0..n:
    for c in 0..m:
        let v = Int(g[Row(r), Col(c)]) - 96
        rh1[r, c + 1] = (rh1[r, c] * BR1 + v) % MOD1
        rh2[r, c + 1] = (rh2[r, c] * BR2 + v) % MOD2

var hashes: List<Int> = []
var ch1: [0..=n]Int = 0
var ch2: [0..=n]Int = 0
let pk_r1 = pr1[k]
let pk_r2 = pr2[k]
let pk_c1 = pc1[k]
let pk_c2 = pc2[k]

for c in 0..=m - k:
    ch1[0] = 0
    ch2[0] = 0
    for r in 0..n:
        var rv1 = (rh1[r, c + k] - rh1[r, c] * pk_r1) % MOD1
        if rv1 < 0:
            rv1 += MOD1
        var rv2 = (rh2[r, c + k] - rh2[r, c] * pk_r2) % MOD2
        if rv2 < 0:
            rv2 += MOD2
        ch1[r + 1] = (ch1[r] * BC1 + rv1) % MOD1
        ch2[r + 1] = (ch2[r] * BC2 + rv2) % MOD2
    for r in 0..=n - k:
        var hv1 = (ch1[r + k] - ch1[r] * pk_c1) % MOD1
        if hv1 < 0:
            hv1 += MOD1
        var hv2 = (ch2[r + k] - ch2[r] * pk_c2) % MOD2
        if hv2 < 0:
            hv2 += MOD2
        hashes.push(hv1 * MOD2 + hv2)

sort(mut hashes)
var distinct = 0
for i in 0..hashes.len:
    if i == 0 or hashes[i] != hashes[i - 1]:
        distinct += 1
print(distinct)

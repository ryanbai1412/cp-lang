use std.rand.Rng

const MAX_V = 5010
type Z = Mod<998_244_353>

var mu: [0..=MAX_V]Z = 0
mu[1] = 1
for i in 1..=MAX_V:
    if mu[i] == 0: continue
    for j in (2 * i..=MAX_V).step(i): mu[j] -= mu[i]

var rng = Rng.new(48)

@!multitest
let n: Int = read()
let raw: [0..n]Int = read()
let g = fold(gcd, raw)
let a = [x / g for x in raw]
let cnt1 = count(x == 1 for x in a)
if cnt1 > 0:
    print(n - cnt1)
    return

let vmax = a.max()!
var freq: [0..=vmax]Z = 0
for x in a: freq[x] += rng.int(1, Z.MOD - 1)

var freq_mult: [0..=vmax]Z = 0
for i in 1..=vmax:
    for j in (i..=vmax).step(i): freq_mult[i] += freq[j]

var freq_prod = [0..=vmax] i => mu[i]
var ans = 1
while true:
    var cnt: Z = 0
    for i in 1..=vmax:
        freq_prod[i] *= freq_mult[i]
        cnt += freq_prod[i]
    if cnt != 0: break
    ans += 1
print(ans - 1 + n - 1)

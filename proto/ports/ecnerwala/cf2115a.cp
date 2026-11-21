use std.rand.Rng
type Z = Mod<998_244_353>
let t: Int = read()
var cases: List<List<Int>> = []
var lim = 2
for _ in 0..t:
    let n: Int = read()
    let a: List<Int> = [read::<Int>() for _ in 0..n]
    for x in a: lim max= x
    cases.push(a)
var mu: [0..(lim + 1)]Z = 0
mu[1] = 1
for i in 1..=lim:
    if mu[i] == 0: continue
    var j = i + i
    while j <= lim:
        mu[j] -= mu[i]
        j += i
var rng = Rng.new(48)
for a0 in cases:
    let n = a0.len
    var g = 0
    for x in a0: g = gcd(g, x)
    var a: List<Int> = []
    var vmax = 1
    var cnt1 = 0
    for x in a0:
        let y = x / g
        a.push(y)
        vmax max= y
        if y == 1: cnt1 += 1
    if cnt1 > 0:
        print(n - cnt1)
        continue
    var freq: [0..(vmax + 1)]Z = 0
    for x in a:
        freq[x] += Z(rng.int(1, 998_244_352))
    var freq_mult: [0..(vmax + 1)]Z = 0
    for i in 1..=vmax:
        var j = i
        while j <= vmax:
            freq_mult[i] += freq[j]
            j += i
    var freq_prod: [0..(vmax + 1)]Z = 0
    for i in 1..=vmax: freq_prod[i] = mu[i]
    var ans = 1
    while true:
        var cnt: Z = 0
        for i in 1..=vmax:
            freq_prod[i] *= freq_mult[i]
            cnt += freq_prod[i]
        if cnt != 0: break
        ans += 1
    print((ans - 1) + (n - 1))

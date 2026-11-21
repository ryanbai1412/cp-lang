type Z = Mod<998_244_353>

fn fold(x: Int, half: Int) -> Int = if x >= half: 2 * half - x else: x

@! multitest
let (n, m): (Int, Int) = read()
let segs: [0..n](Int, Int) = read()
let full = 1 << m
var ans: [0..full]Z = 0
ans[0] = 1
for (l, r) in segs: ans[0] *= r - l + 1

for q in 1..=m:
    let sz = 1 << (m - q)
    let half = 1 << (q - 1)
    let low = 2 * half - 1
    var va: [0..sz]Z = 1
    var vb: [0..sz]Z = 1
    var high_xor = 0
    for (l, last) in segs:
        let r = (last + 1) & (full - 1)
        let (vl, vr) = (fold(l & low, half), fold(r & low, half))
        high_xor ^= r >> q
        let hx = (l ^ r) >> q
        va[hx] *= vr - vl
        vb[hx] *= vr + vl
    for z in 0..m - q:
        let bit = 1 << z
        for i in (0..sz).step(2 * bit):
            for j in i..i + bit:
                let k = j + bit
                (va[j], vb[j], va[k], vb[k]) = (
                    va[j] * va[k],
                    vb[j] * vb[k],
                    va[j] * vb[k],
                    vb[j] * va[k],
                )
    for i in 0..sz:
        ans[(i << q) + half] = if popcount(high_xor & i) % 2 == 1: -va[i] else: va[i]

for q in 0..m:
    let bit = 1 << q
    for i in (0..full).step(2 * bit):
        for j in i..i + bit:
            (ans[j], ans[j + bit]) = (ans[j] + ans[j + bit], ans[j] - ans[j + bit])

var xor = 0
var coeff = Z(full).inv
for x in ans:
    xor ^= (x * coeff).v
    coeff *= 2
print(xor)

const MOD = 998244353

@! multitest
let n: Int = read()
let m: Int = read()
var segs: List<(Int, Int)> = []
for _ in 0..n:
    let (l, r): (Int, Int) = read()
    segs.push((l, r))
let full = 1 << m
var ans: [0..full]Mod<MOD> = 0
var v0: Mod<MOD> = 1
for (l, r) in segs:
    v0 *= r - l + 1
ans[0] = v0
for q in 1..=m:
    let sz = 1 << (m - q)
    var va: [0..sz]Mod<MOD> = 1
    var vb: [0..sz]Mod<MOD> = 1
    var tot_hr = 0
    let mask_m = full - 1
    let mask_q = (1 << q) - 1
    let half_q = 1 << (q - 1)
    for (l, r0) in segs:
        let r = (r0 + 1) & mask_m
        var vr = r & mask_q
        if vr >= half_q:
            vr -= 2 * (vr - half_q)
        var vl = l & mask_q
        if vl >= half_q:
            vl -= 2 * (vl - half_q)
        tot_hr ^= r >> q
        let hx = (l ^ r) >> q
        let d0: Mod<MOD> = vr - vl
        let d1: Mod<MOD> = vr + vl
        va[hx] *= d0
        vb[hx] *= d1
    for z in 0..m - q:
        let bit = 1 << z
        var i = 0
        while i < sz:
            for j in i..i + bit:
                let aa = va[j]
                let ab = vb[j]
                let ba = va[j + bit]
                let bb = vb[j + bit]
                va[j] = aa * ba
                vb[j] = ab * bb
                va[j + bit] = aa * bb
                vb[j + bit] = ab * ba
            i += bit * 2
    for i in 0..sz:
        let sign = if (popcount(tot_hr & i) & 1) == 1: -1 else: 1
        let ms: Mod<MOD> = sign
        ans[(i << q) + half_q] = va[i] * ms
for q in 0..m:
    let bit = 1 << q
    var i = 0
    while i < full:
        for j in i..i + bit:
            let x = ans[j]
            let y = ans[j + bit]
            ans[j] = x + y
            ans[j + bit] = x - y
        i += bit * 2
let inv_full = (Mod<MOD>(full)).inv()
var final_ans = 0
var coeff: Mod<MOD> = 1
for i in 0..full:
    let val = ans[i] * inv_full * coeff
    final_ans ^= val.v
    coeff *= 2
print(final_ans)

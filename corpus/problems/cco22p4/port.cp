// CCO '22 P4 — Good Groups / Binary Tree Parking (today form for proto/cpc)
// original: corpus/submissions/4801884.cpp
let n: 1..=4000 = read()
let m: 0..=4000 = read()

let MOD = 1_000_000_007

fn bpow(base0: Int, exp0: Int) -> Int:
    var ans = 1
    var base = base0 % MOD
    var exp = exp0
    while exp > 0:
        if exp % 2 == 1: ans = (ans * base) % MOD
        base = (base * base) % MOD
        exp = exp / 2
    ans

var fact: List<Int> = [1]
var ifact: List<Int> = [1]
for i in 1..=n:
    let f = (fact[i - 1] * i) % MOD
    fact.push(f)
    ifact.push(bpow(f, MOD - 2))

fn choose(nn: Int, kk: Int) -> Int:
    fact[nn] * ifact[kk] % MOD * ifact[nn - kk] % MOD

fn arrange(nn: Int, kk: Int) -> Int:
    fact[nn] * ifact[nn - kk] % MOD

domain V = 0..n
domain K = 0..=n

var here: [V]Int = 0
for _ in 0..m:
    let u: 1..=n = read()
    here[V(u - 1)] += 1

var lc: [V]Int = -1
var rc: [V]Int = -1
for i in V:
    let a: Int = read()
    let b: Int = read()
    lc[i] = a - 1
    rc[i] = b - 1

var is_right: [V]Bool = false
var sub_sz: [V]Int = 0
var sub_sum: [V]Int = 0
var spare: [V]Int = 0

fn prep(u: Int):
    let vu = V(u)
    sub_sz[vu] = 1
    sub_sum[vu] = here[vu]
    let l = lc[vu]
    if l != -1:
        is_right[V(l)] = false
        prep(l)
        sub_sz[vu] += sub_sz[V(l)]
        sub_sum[vu] += sub_sum[V(l)]
    let r = rc[vu]
    if r != -1:
        is_right[V(r)] = true
        prep(r)
        sub_sz[vu] += sub_sz[V(r)]
        sub_sum[vu] += sub_sum[V(r)]
    spare[vu] = sub_sz[vu] - sub_sum[vu]

prep(0)

var bad = false
for u in V:
    if spare[u] < 0: bad = true

var memo: [V, K]Int = -1

fn dp(u: Int, k: Int) -> Int:
    let vu = V(u)
    if k > spare[vu]: return 0
    let l0 = lc[vu]
    let r0 = rc[vu]
    let ccnt = Int(l0 != -1) + Int(r0 != -1)
    if ccnt == 0: return 1
    let cached = memo[vu, K(k)]
    if cached != -1: return cached
    var ans = 0
    if ccnt == 1:
        let ch = if l0 != -1: l0 else: r0
        let nk = if k == spare[vu]: k + here[vu] - 1 else: k + here[vu]
        ans = arrange(k + here[vu], here[vu]) * dp(ch, nk) % MOD
    else:
        let first = if is_right[vu]: r0 else: l0
        let second = if is_right[vu]: l0 else: r0
        let sf = spare[V(first)]
        let ss = spare[V(second)]
        let hu = here[vu]
        for i in 0..=hu:
            let bl = i
            let br = hu - i
            let al = min(k, sf - bl)
            let ar = k - al
            if al >= 0:
                let xr = if al + bl == sf and ar + br > ss: ar + br - 1 else: ar + br
                let term = (
                    choose(hu, i) * arrange(al + bl, bl) % MOD * arrange(ar + br, br) % MOD * dp(
                        first,
                        al + bl,
                    ) % MOD * dp(second, xr) % MOD
                )
                ans = (ans + term) % MOD
    memo[vu, K(k)] = ans
    ans

if bad:
    print(0)
else:
    print(dp(0, 0))

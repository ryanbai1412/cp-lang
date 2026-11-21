let (n, m): (Int, Int) = read()
var cc: [0..n]Int = 0
var ww: [0..n]Int = 0
var cmax = 1
for i in 0..n:
    let (c, w): (Int, Int) = read()
    cc[i] = c
    ww[i] = w
    cmax max= c
let smax = cmax * cmax
var adj: [0..n]List<Int> = []
for _ in 0..m:
    let (u, v): (Int, Int) = read()
    adj[u - 1].push(v - 1)
let q: Int = read()
var qp: [0..q]Int = 0
var qr: [0..q]Int = 0
for i in 0..q:
    let (p, r): (Int, Int) = read()
    qp[i] = p - 1
    qr[i] = r
var ans: [0..q]Int = 0
var dp0: [0..n, 0..(smax + 1)]Int = 0
for i in 0..n:
    let ci = cc[i]
    let wi = ww[i]
    for j in 0..=(smax - ci):
        dp0[i, j + ci] max= dp0[i, j] + wi
    for k in adj[i]:
        for j in 0..=smax:
            dp0[k, j] max= dp0[i, j]
for qi in 0..q:
    ans[qi] max= dp0[qp[qi], min(qr[qi], smax)]
for mid in 0..n:
    let modv = cc[mid]
    let modw = ww[mid]
    var vis: [0..n]Bool = false
    var dp: [0..n, 0..modv]Int = 0
    vis[0] = true
    for i in 0..n:
        if not vis[i]: continue
        if ww[i] * modv <= modw * cc[i]:
            let g = gcd(cc[i], modv)
            let steps = 2 * (modv / g)
            for st in 0..g:
                var j = st
                for _ in 0..=steps:
                    let nj = j + cc[i]
                    dp[i, nj % modv] max= dp[i, j] + ww[i] - (nj / modv) * modw
                    j = nj % modv
        for k in adj[i]:
            if i < mid and mid < k: continue
            vis[k] = true
            for j in 0..modv:
                dp[k, j] max= dp[i, j]
    for qi in 0..q:
        let p = qp[qi]
        let r = qr[qi]
        if p >= mid and vis[p] and r >= smax:
            ans[qi] max= dp[p, r % modv] + (r / modv) * modw
for qi in 0..q:
    print(ans[qi])

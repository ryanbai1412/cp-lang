@!multitest
let (n, k): (2..=2e5, 1..=1500) = read()
let a: [0..n]1..=1e9 = read()
var ev: List<(Int, Int, Int, Int)> = []
for i in 0..n:
    for g in 1..=k + 1:
        let j = i + g
        if j >= n: break
        let diff = a[j] - a[i]
        if diff % g == 0:
            let d = diff / g
            ev.push((d, a[i] - d * i, i, j))
ev.sort()
var best = 2
var s = 0
while s < ev.len:
    var e = s
    var ps: List<Int> = []
    while e < ev.len and ev[e].0 == ev[s].0 and ev[e].1 == ev[s].1:
        ps.push(ev[e].2)
        ps.push(ev[e].3)
        e += 1
    ps.sort()
    let qs = ps.deduped()
    var l = 0
    for r in 0..qs.len:
        while (qs[r] - qs[l] + 1) - (r - l + 1) > k: l += 1
        best = max(best, r - l + 1)
    s = e
print(min(n, best + k))

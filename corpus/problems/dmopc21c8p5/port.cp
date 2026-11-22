// Runnable port (proto/cp.py lib-run) of dmopc21c8p5.cp
// original: corpus/submissions/4533533.cpp

let (n, k): (Int, Int) = read()
domain K = 1..=k
let a: [K]Int = read()

let base = max(2 * k + 2, 4)
domain Small = 0..=base

var v: List<Int> = [0]
for i in 1..k:
    v.push(a[K(i + 1)] + (i - 1) * a[K.first])

var sm: [Small]?Int = [Small] _ => none
sm[Small(0)] = 0
for i in 1..=base:
    var best: ?Int = none
    let jmax = min(i, k - 1)
    for j in 1..=jmax:
        if let prev = sm[Small(i - j)]:
            let cand = prev + v[j]
            if best == none or cand < best!:
                best = cand
    sm[Small(i)] = best

var cache = Map<Int, Int>()

fn dp(x: Int) -> Int:
    if x <= base: return sm[Small(x)]!
    if x in cache: return cache[x]
    let lo = x / 2 - k / 2 - 1
    var best: ?Int = none
    for i in lo..=(lo + k):
        let cand = dp(i) + dp(x - i)
        if best == none or cand < best!:
            best = cand
    cache[x] = best!
    best!

print(dp(n - 2) + 2 * a[K.first])

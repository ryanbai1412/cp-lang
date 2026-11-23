let n: 2..=16 = read()
domain T = 0..n
let d: [T, T]0..=1e6 = read()
var best: [0..1 << n, T]?Int = None
best[1, T(0)] = Some(0)
for mask in 1..1 << n:
    for u in T if mask.bit(u.ord):
        let cur = best[mask, u] else: continue
        for v in T if not mask.bit(v.ord):
            best[mask | (1 << v.ord), v] min= cur + d[u, v]
let full = (1 << n) - 1
var tour: ?Int = None
for u in T:
    if let c = best[full, u]: tour min= c + d[u, T(0)]
print(tour!)

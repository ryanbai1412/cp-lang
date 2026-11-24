let n: 2..=16 = read()
domain T = 0..n
let d: [T, T]0..=1e6 = read()
var best: [T.subsets, T]?Int = None
best[Subset<T>.single(T(0)), T(0)] = Some(0)
for mask in T.subsets:
    for u in mask.items:
        let cur = best[mask, u] else: continue
        for v in mask.complement().items:
            best[mask.with(v), v] min= cur + d[u, v]
var tour: ?Int = None
for u in T:
    if let c = best[Subset<T>.full, u]: tour min= c + d[u, T(0)]
print(tour!)

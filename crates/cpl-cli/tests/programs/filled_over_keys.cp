// `[e].filled(x)` reads `e` as `[e]T` does: `[a.keys]` for an `a: [K]T` is
// `[K]`, and `[w.keys]` for a slice `w` is keyed by `w`'s keys (R04b.1.5).
fn marks<K>(a: [K]Int) -> [K]?K:
    var res: [K]?K = [a.keys].filled(None)
    for i in a.keys:
        if a[i] > 1: res[i] = i
    res
let a = [3, 1, 4, 1]
let m = marks(a)
print(m[0]!, m[1] ?? 9, m[2]!)
let w = [1, 2, 3, 4][1..]
var t = [w.keys].filled(7)
t[2] = 1
print(t[1], t[2], t.len)
let n = 2
print([n].filled(5))

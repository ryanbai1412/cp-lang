// original: 4583882 (cco16p1) -- first draft, written from docs/one-page-cheat-sheet.md only; does not build on proto/cpc
let (n, m, k): (Int, Int, Int) = read()
domain V = 1..=n
let es: [0..m](V, V) = read()
var dsu = Dsu<V>()
for (u, v) in es: dsu.unite(u, v)
var ec: [V]Int = 0
for (u, v) in es: ec[dsu.find(u)] += 1
var ans = 0
var bad = 0
for v in V:
    if dsu.find(v) != v: continue
    let s = dsu.size(v)
    let x = s / k
    ans += x
    if s <= k: continue
    let cyc = ec[v] == s
    if cyc: bad += if s % k == 0: x else: x + 1
    else: bad += if s % k == 0: x - 1 else: x
print(ans * k, bad)

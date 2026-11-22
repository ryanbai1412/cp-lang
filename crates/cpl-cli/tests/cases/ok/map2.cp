// cp: 0.1 cc468f2+dirty
let n: 1..=10 = read()
domain V = 1..=n
let a: [V]1..=5 = read()
var first_at = Map<Int, V>()
var cnt: Map<V, Int> = []
for v in V:
    if not (a[v] in first_at):
        first_at[a[v]] = v
for (x, v) in first_at:
    cnt[v] = x
print(cnt.len, cnt.get(V.first!) ?? -1, cnt.get(V.last!) ?? -1)
for (v, x) in cnt:
    print(v, x, cnt[v])
cnt.remove(V.first!)
print(cnt.len, yes(V.first! in cnt))

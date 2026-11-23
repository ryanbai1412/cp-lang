let a = [3, 1, 3, 2, 3]
var cnt: Map<Int, Int> = Map()
for x in a:
    cnt[x] = (cnt.get(x - 1) ?? 0) + 1
var best: Map<Int, Int> = Map()
for x in a:
    best[x] = (best.get(x) ?? 5) + 1
print(cnt[3], best[3])
var weight: Map<Int, Int> = Map()
for x in a:
    weight[x] += x
var seen: Map<Int, Int> = Map()
for x in a:
    seen[x] += 1
print(weight[3], seen.first)
var shown: Map<Int, Int> = Map()
for x in a:
    shown[x] += 1
print(shown)
var dense: [0..=5]Int = 0
for x in a:
    dense[x] += 1
print(dense[3])

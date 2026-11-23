let a = [3, 1, 3, 2, 3]
var cnt: Map<Int, Int> = Map()
for x in a:
    cnt[x] = (cnt.get(x - 1) ?? 0) + 1
var best: Map<Int, Int> = Map()
for x in a:
    best[x] = (best.get(x) ?? 5) + 1
print(cnt[3], best[3])

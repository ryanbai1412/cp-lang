let a = [3, 1, 3, 2, 3]
var cnt: Map<Int, Int> = Map()
for x in a:
    cnt[x] = (cnt.get(x) ?? 0) + 1
print(cnt[3])

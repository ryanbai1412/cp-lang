let a = [3, 1, 3, 2, 3]
var cnt: Map<Int, Int> = Map()
for x in a:
    cnt[x] = (cnt.get(x) ?? 0) + 1
print(cnt[3])
var freq: Map<Int, Int> = Map()
for x in a:
    freq[x] += 1
let odd = [c for (k, c) in freq.items if k % 2 == 1]
print(odd.len)

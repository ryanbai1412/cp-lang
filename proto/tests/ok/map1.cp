// cp: 0.1 cc468f2+dirty
let n: 1..=10 = read()
let xs: [n]-5..=5 = read()
var cnt: Map<Int, Int> = []
var seen = Set<Int>()
for x in xs:
    if x in cnt:
        cnt[x] += 1
    else:
        cnt[x] = 1
    seen.insert(x * x)
for (k, v) in cnt:
    print(k, v)
print(seen.len, seen.min() ?? -1, cnt.get(100) ?? 0)

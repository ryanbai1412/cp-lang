let q: 1..=200000 = read()
var cnt: Map<Int, Int> = Map()
var size = 0
for _ in 0..q:
    let (t, x): (1..=2, 1..=1000000000) = read()
    if t == 1:
        cnt[x] += 1  // expect: missing map_as_multiset
        size += 1
    elif cnt[x] > 0:
        cnt[x] -= 1
        size -= 1
print(size)

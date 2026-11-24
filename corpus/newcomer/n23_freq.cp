let n: 1..=200000 = read()
let a: [0..n]Int = read()
var cnt: Map<Int, Int> = Map()
for x in a:
    if x in cnt:  // expect: contains_then_add
        cnt[x] += 1
    else:
        cnt[x] = 1
var best = 0
for (k, v) in cnt.items:
    best = max(best, v)  // expect: chmin_chmax_if
print(best)

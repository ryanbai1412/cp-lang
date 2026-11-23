let n: 1..=200000 = read()
let a: [0..n]Int = read()
var cnt: Map<Int, Int> = Map()
for x in a:
    if x in cnt:  // expect: missing map-contains-then-increment
        cnt[x] += 1
    else:
        cnt[x] = 1
var best = 0
for (k, v) in cnt.items:  // expect: loop_reduction
    best = max(best, v)
print(best)

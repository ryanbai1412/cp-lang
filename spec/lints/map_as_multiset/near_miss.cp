let n: 1..=200000 = read()
let a: [0..n]Int = read()
var cnt: Map<Int, Int> = Map()
for x in a:
    cnt[x] += 1
print(cnt.len)

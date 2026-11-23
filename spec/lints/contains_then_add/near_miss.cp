let n: 1..=100 = read()
let a: [0..n]Int = read()
var cnt: Map<Int, Int> = Map()
for x in a:
    if x in cnt:
        cnt[x] += 1
    else:
        cnt[x] = 2
print(cnt)

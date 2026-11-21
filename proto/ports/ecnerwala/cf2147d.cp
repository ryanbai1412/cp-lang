@!multitest
let n: Int = read()
let a: [0..n]Int = read()

var freq: Map<Int, Int> = Map()
for x in a: freq[x] += 1

let base = sum(x / 2 for x in a)
let extras = [cnt for (x, cnt) in freq.items if x % 2 == 1].sorted(by: c => -c)
let a0 = base + sum(extras[z] for z in (0..extras.len).step(2))
let a1 = base + sum(extras[z] for z in (1..extras.len).step(2))
print(a0, a1)

let n: 1..=10 = read()
let all: [0..n]Int = read()
let xs = compress(all)
var cnt: [xs.D]Int = 0
for v in all: cnt[xs.at(v)] += 1
let top = xs.at(50)
print(xs.D.count, cnt[top], xs.at(10).ord, xs.at(all[0]).ord, xs.at(30).ord, xs[top], xs.first_ge(20)!.ord, xs.first_ge(31)!.ord)

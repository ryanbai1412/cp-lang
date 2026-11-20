let n: 1..=10 = read()
let all: [0..n]Int = read()
let xs = compress(all)
var cnt: [xs.D]Int = 0
for v in all: cnt[xs.at(v)] += 1
let top = xs.at(50)
print(xs.D.K, cnt[top], xs.at(10).to_idx, xs.at(all[0]).to_idx, xs.at(30).to_idx, xs[top], xs.first_ge(20)!.to_idx, xs.first_ge(31)!.to_idx)

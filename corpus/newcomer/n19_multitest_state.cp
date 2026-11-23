var cnt: [0..=200000]Int = 0
@!multitest
let n: 1..=200000 = read()
let a: [0..n]1..=200000 = read()
var best = 0
for x in a:
    cnt[x] += 1  // expect: state_above_multitest
    best max= cnt[x]
for x in a:
    cnt[x] = 0  // expect: state_above_multitest
print(best)

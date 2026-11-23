// Codeforces 1917C Watering an Array
// in: t; per test n k d, then a (n ints), v (k ints); out: per test, the maximum score after d days
@!multitest
let n: 1..=2000 = read()
let k: 1..=1e5 = read()
let d: k..=1e9 = read()
domain I = 1..=n
var a: [I]Int = read()
let v = read<[k]1..=n>()

// Add on the first `adds` days, reset on the next, then alternate add and reset.
var best = 0
for adds in 0..=2 * n:
    if adds >= d: break
    let now = I.count(j => a[j] == Int(j))
    best max= now + (d - adds - 1) / 2
    for j in 1..=v[adds % k]: a[I(j)] += 1
print(best)

// 1923B Monsters Attack!
// in: t; per test n k, a_1..a_n (health), x_1..x_n (positions). out: YES/NO per test
@!multitest
let n, k: Int = read()
let a: [n]Int = read()
let x: [n]Int = read()
var at: [n + 1]Int = 0
for i in 0..n:
    at[abs(x[i])] += a[i]
let pre = at.scan(+)
print(yes((1..=n).all(d => pre[d + 1] <= k * d)))

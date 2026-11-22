// prefix sums, with the loop one step too far
let n: 1..=5 = read()
let a: [0..n]Int = read()
var pre: [0..=n]Int = 0
for i in 0..=n:
    pre[i + 1] = pre[i] + a[i]
print(pre[n])

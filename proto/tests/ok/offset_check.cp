let n: 1..=2e5 = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var pre: [V.bounds]Int = 0
for i in V if let j = i + 1:
    pre[i.r] = a[j]
    print(pre[i.r])

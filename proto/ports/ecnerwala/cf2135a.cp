@!multitest
let n: Int = read()
let a: [0..n]Int = read()
var inds: [0..=n]List<Int> = []
for (i, x) in a.items: inds[x].push(i)

var dp: [0..=n]Int = 0
for i in (0..n).rev:
    dp[i] max= dp[i + 1]
    let x = a[i]
    if inds[x].len >= x:
        dp[inds[x][inds[x].len - x]] max= dp[i + 1] + x
    inds[x].pop()
print(dp[0])

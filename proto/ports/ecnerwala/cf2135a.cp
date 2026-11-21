@!multitest
let n: Int = read()
let a: List<Int> = [read::<Int>() for _ in 0..n]
var inds: [0..=n]List<Int> = []
for i in 0..n: inds[a[i]].push(i)
var dp: [0..=n]Int = 0
for i in (0..n).rev():
    dp[i] max= dp[i + 1]
    let x = a[i]
    if inds[x].len >= x:
        let j = inds[x][inds[x].len - x]
        dp[j] max= dp[i + 1] + x
    inds[x].pop()
print(dp[0])

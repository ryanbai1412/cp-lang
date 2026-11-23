// CF 1918D Blocking Elements: binary search on the answer x; a cheapest block set whose
// gaps all sum to at most x is a sliding-window-min DP over the previous block.
@!multitest
let n: 1..=1e5 = read()
let a = read<[n]1..=1e9>()
var w: [n + 2]Int = 0
for i in 0..n:
    w[i + 1] = a[i]
let seg = Prefix(w)

fn feasible(x: Int) -> Bool:
    var dp: [n + 2]Int = 0
    var best = Window(min)
    var lo = 0
    best.push(0)
    for i in 1..n + 2:
        while seg.fold(lo + 1..i) > x:
            best.pop()
            lo += 1
        dp[i] = w[i] + best.all!
        best.push(dp[i])
    dp[n + 1] <= x

print((0..=a.sum()).first_true(feasible)!)

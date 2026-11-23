// CF 1994C Hungry Games: count (l, r) whose final toxicity g is non-zero.
// From l, the longest piece with sum <= x ends just before the mushroom that resets g;
// after a reset the walk restarts at the next mushroom, so dp[l] = len + dp[end + 1].

@!multitest
let (n, x): (1..=2e5, 1..=1e9) = read()
let a = read<[n]Int>()
// end[l]: the first mushroom left out of the longest piece from l with sum <= x (two pointers)
var end: [n]Int = 0
var w = Window(+)
var r = 0
for l in 0..n:
    while r < n and (w.all ?? 0) + a[r] <= x:
        w.push(a[r])
        r += 1
    end[l] = r
    if r > l: w.pop() else: r = l + 1
var dp: [n + 1]Int = 0                 // dp[l]: good r for this l; dp[n] = 0
for l in (0..n).rev:
    let e = end[l]
    dp[l] = if e == n: n - l else: (e - l) + dp[e + 1]
print(dp.sum())

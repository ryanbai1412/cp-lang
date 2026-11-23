// CF 1932F Feed Cats: dp over prefixes; feeding at x takes every cat covering x and forbids
// any earlier feeding point at or after lo[x], the leftmost start among cats covering x.
@!multitest
let n: 1..=1e6 = read()
let m: 1..=2e5 = read()
domain P = 1..=n
let cats = read<[m](l: P, r: P)>()
var start = [0] * (n + 2)
var min_l: [P]Int = n + 1
for c in cats:
    start[Int(c.l)] += 1
    start[Int(c.r) + 1] -= 1
    min_l[c.r] min= Int(c.l)
var dp = [0] * (n + 1)
var covering = 0
var lo_suffix = n + 1
var lo: [P]Int = 0
for x in P.rev:
    lo_suffix min= min_l[x]
    lo[x] = min(lo_suffix, Int(x))
for x in P:
    let i = Int(x)
    covering += start[i]
    dp[i] = max(dp[i - 1], dp[lo[x] - 1] + covering)
print(dp[n])

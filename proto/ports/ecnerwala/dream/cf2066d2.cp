// Codeforces 2066 D2: Club of Young Aircraft Builders (hard version).
// Process floors 1..n - 1 from bottom to top; floor n takes the last c launches.
// If floors < f have taken `used` launches, floor f sees only the remaining
// launches and stops after c of them, so all of floor f's launches must lie in
// the prefix of length `used + c`. That requires `used + c > last[f]`, leaves
// `open[used + c] + fixed_below - used` free slots in that prefix, and choosing
// k launches for floor f multiplies by C(avail, k - fixed[f]).
@!multitest
type M = Mod<1e9 + 7>

let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
domain Floor = 1..=n
domain Time = 1..=m
let a: [Time]?Floor = read(none: 0)

let open = a.map(x => Int(x == none)).prefix_sums(at: .cuts)
var dp: [Time.cuts]M = 0
dp[Time.cuts.first] = 1
var fixed_below = 0

for f in Floor.init:
    let fixed = a.count(x => x == f)
    let min_used = match Time.last(t => a[t] == f):
        some(t) => max(0, (t + 1) - c)
        none => 0
    dp = [Time.cuts] next => 0
    for used in Time.cuts if used >= min_used and used <= m - c and dp[used] != 0:
        let avail = open[used + c] + fixed_below - used
        for k in fixed..=min(c, m - c - used):
            next[used + k] += dp[used] * M.choose(avail, k - fixed)
    fixed_below += fixed

print(dp[m - c])

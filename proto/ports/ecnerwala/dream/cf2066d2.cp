// Codeforces 2066 D2: Club of Young Aircraft Builders (hard version) (dream).
// Process floors 1..n - 1 from bottom to top; floor n takes the last c launches.
// If floors < f have taken `used` launches, floor f sees only the remaining
// launches and stops after c of them, so all of floor f's launches must lie in
// the prefix of length `used + c`. That requires `used + c > last[f]`, leaves
// `open[used + c] + fixed_below - used` free slots in that prefix, and choosing
// k launches for floor f multiplies by C(avail, k - fixed[f]).
type M = Mint(mod: 1e9 + 7)
let comb = Comb<M>(100)

--- multitest ---
let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
domain Floor = 1..=n
domain Time = 1..=m
let a: [Time]?Floor = read(none: 0)

let open = prefix_sums([Int(x == none) for x in a])
var dp: [0..=m - c]M = 0
dp[0] = 1
var fixed_below = 0

for f in Floor.init:
    let fixed = count(Time, t => a[t] == f)
    let min_used = match Time.last(where: t => a[t] == f):
        some(t) => max(0, t.ord + 1 - c)
        none => 0
    var ndp: [0..=m - c]M = 0
    for used in min_used..=m - c where dp[used] != 0:
        let avail = open[used + c] + fixed_below - used
        for k in fixed..=min(c, m - c - used):
            ndp[used + k] += dp[used] * comb.choose(avail, k - fixed)
    dp = ndp
    fixed_below += fixed

print(dp[m - c])

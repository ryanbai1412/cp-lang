// Codeforces 2066 D2: Club of Young Aircraft Builders (hard version).
// Process floors 1..n - 1 from bottom to top; floor n takes the last c launches.
// If floors < f have taken `used` launches (in 0..=m - c), floor f sees only
// the remaining launches and stops after c of them, so all of floor f's
// launches must lie in the prefix of length `used + c`. That requires
// `used + c >= last_pos[f]` (1-based position of floor f's last fixed launch),
// leaves `avail = open[used + c] + fixed_below - used` free slots in that
// prefix, and choosing k launches for floor f multiplies by
// C(avail, k - fixed[f]).

type M = Mod<1_000_000_007>
let comb = Comb<M>(100)

@!multitest
let (n, c, m): (1..=100, 1..=100, c..=n * c) = read()
domain Floor = 1..=n
domain Time = 1..=m
domain Used = 0..=m - c
let a: [Time]?Floor = read(none: 0)

let open = prefix([Time] t => Int(a[t] == none))
var dp: [Used]M = 0
dp[Used.first] = 1
var fixed_below = 0

for f in Floor.range.drop_last():
    let fixed = a.count(x => x == f)
    let min_used = match Time.range.last(t => a[t] == f):
        some(t) => Used(max(0, t.ord + 1 - c))
        none => Used.first
    var next: [Used]M = 0
    for used in min_used..=Used.last if dp[used] != 0:
        let avail = open.sum(..=Time(used.ord + c)) + fixed_below - used.ord
        for k in fixed..=min(c, m - c - used.ord):
            next[Used(used.ord + k)] += dp[used] * comb.choose(avail, k - fixed)
    dp = next
    fixed_below += fixed

print(dp[Used.last])

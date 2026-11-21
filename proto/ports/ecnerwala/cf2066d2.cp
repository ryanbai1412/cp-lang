// Codeforces 2066 D2: Club of Young Aircraft Builders (hard version).
// Process floors 1..n - 1 from bottom to top; floor n takes the last c launches.
// If floors < f have taken `used` launches, floor f sees only the remaining
// launches and stops after c of them, so all of floor f's launches must lie in
// the prefix of length `used + c`. That requires `used + c > last[f]`, leaves
// `open[used + c] + fixed_below - used` free slots in that prefix, and choosing
// k launches for floor f multiplies by C(avail, k - fixed[f]).
use std.prefix.prefix1

const P = 1_000_000_007
type M = Mod<P>
let comb = Comb<M>(100)

let t: 1..=10_000 = read()
repeat t:
    let n: 1..=100 = read()
    let c: 1..=100 = read()
    let m: c..=n * c = read()
    domain Floor = 1..=n
    domain Time = 1..=m
    let a: [Time]?Floor = read(none: 0)

    let open = prefix1([Time] tm => Int(a[tm] == none))
    var last_pos: [Floor]?Time = none
    for tm in Time:
        if let f = a[tm]: last_pos[f] = tm

    var dp: [0..=m - c]M = 0
    dp[0] = 1
    var fixed_below = 0

    for f in Floor if f != Floor.last:
        let fixed = count(a[tm] == f for tm in Time)
        let min_used = if let lp = last_pos[f]: max(0, lp.ord + 1 - c) else: 0
        var ndp: [0..=m - c]M = 0
        for used in min_used..=m - c if dp[used] != 0:
            let avail = open.sum(..=Time(used + c)) + fixed_below - used
            for k in fixed..=min(c, m - c - used):
                ndp[used + k] += dp[used] * comb.choose(avail, k - fixed)
        dp = ndp
        fixed_below += fixed

    print(dp[m - c])

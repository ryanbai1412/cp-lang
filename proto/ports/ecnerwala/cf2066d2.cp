// Codeforces 2066 D2: Club of Young Aircraft Builders (hard version).
// Process floors 1..n - 1 from bottom to top; floor n takes the last c launches.
// If floors < f have taken `used` launches, floor f sees only the remaining
// launches and stops after c of them, so all of floor f's launches must lie in
// the prefix of length `used + c`. That requires `used + c > last[f]`, leaves
// `open[used + c] + fixed_below - used` free slots in that prefix, and choosing
// k launches for floor f multiplies by C(avail, k - fixed[f]).
type M = Mod<1_000_000_007>
let comb = Comb<1_000_000_007>(100)

let t: 1..=10_000 = read()
for _ in 0..t:
    let (n, c, m): (1..=100, 1..=100, 1..=10_000) = read()
    domain Floor = 1..=n
    domain Time = 1..=m
    let raw: [Time](0..=100) = read()

    var fixed: [Floor]Int = 0
    var last_pos: [Floor]?Time = none
    var open: [0..=m]Int = 0
    for tm in Time:
        open[tm.ord + 1] = open[tm.ord] + Int(raw[tm] == 0)
        if raw[tm] > 0:
            let f = Floor(raw[tm])
            fixed[f] += 1
            last_pos[f] = tm

    var dp: [0..=m - c]M = 0
    dp[0] = 1
    var fixed_below = 0

    for fi in 1..n:
        let f = Floor(fi)
        let min_used = if let lp = last_pos[f]: max(0, lp.ord + 1 - c) else: 0
        var ndp: [0..=m - c]M = 0
        for used in min_used..=m - c:
            let ways = dp[used]
            if ways == 0: continue
            let avail = open[used + c] + fixed_below - used
            for k in fixed[f]..=min(c, m - c - used):
                ndp[used + k] += ways * comb.choose(avail, k - fixed[f])
        dp = ndp
        fixed_below += fixed[f]

    print(dp[m - c])

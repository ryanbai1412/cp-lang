// NOI 2023 D1T2 Guiyang Tree: count valid tree extensions of m new vertices
// with window parameter k, modulo 10^9 + 7. Broken-profile bitmask DP.
// original: corpus/submissions/5723059.cpp
const P = 1000000007
type Z = Mod<P>
let tc_id: 0..=25 = read()
let tcs: 1..=10 = read()

for _ in 0..tcs:
    let n: 1..=30000 = read()
    let m: 0..=3000 = read()
    let k: 0..=10 = read()
    let parents: [n - 1]1..=30000 = read()
    if k == 0:
        var ans = Z(1)
        for i in 0..m: ans *= 2 * (n + i) - 1
        print(ans)
    else:
        let lim = 1 << k
        let half = 1 << (k - 1)
        var pcnt: [0..lim]Int = 0
        for mask in 1..lim: pcnt[mask] = pcnt[mask / 2] + (mask & 1)

        var dp: [0..lim]Z = Z(0)
        dp[0] = 1
        for i in (0..m).rev:
            var next: [0..lim]Z = Z(0)
            for mask in half..lim:
                next[mask] = dp[(mask << 1) ^ lim]
            for mask in 0..half:
                let cn = n + i + pcnt[mask]
                var cur = Z(2 * cn - 1) * dp[mask << 1] + Z(cn - 1) * dp[(mask << 1) | 1]
                var x = mask
                while x > 0:
                    let low = x & (-x)
                    cur += dp[(mask ^ low) << 1]
                    x = x ^ low
                next[mask] = cur
            dp = next
        print(dp[0])

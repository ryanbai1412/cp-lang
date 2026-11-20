// NOI 2023 D1T2 Guiyang Tree: count valid tree extensions of m new vertices
// with window parameter k, modulo 10^9 + 7. Broken-profile bitmask DP.
// original: corpus/submissions/5723059.cpp
const P = 1000000007
type Z = Mod<P>
let tc_id: 0..=25 = read()                                        // WISH P14
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
        domain Win = 0..k
        type Mask = Subset<Win>
        type Layer = [Mask]Z
        var dp: Layer = 0
        dp[Mask.empty] = 1
        for i in (0..m).rev:
            var next: Layer = 0
            for mask in Mask:
                let shifted = mask.shifted(1)                     // WISH P11
                if Win.last in mask:
                    next[mask] = dp[shifted]
                else:
                    let cn = n + i + mask.len
                    var cur = Z(2 * cn - 1) * dp[shifted] + Z(cn - 1) * dp[shifted.with(Win.first)]
                    for bit in mask: cur += dp[mask.without(bit).shifted(1)]   // WISH P11
                    next[mask] = cur
            dp = next
        print(dp[Mask.empty])

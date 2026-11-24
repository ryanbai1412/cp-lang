// Source: NOI 2023 Day 1 P2 (Osmanthus Tree), https://dmoj.ca/problem/noi23p2
// limits: 500 ms, 512 MB
// Problem: a rooted tree T on 1..n (n <= 3e4, parent of i < i) and k <= 10. Count rooted trees
//   T' on n + m nodes (m <= 3000, root 1) where every two original nodes keep their LCA and
//   every pair i, j has an LCA numbered <= max(i, j) + k. Modulo 1e9 + 7; t <= 15 tests.
// Solution: insert labels n + 1, n + 2, ... in order, keeping the tree the labels so far span
//   plus pending branch nodes. A label hung off the middle of an edge makes a new branch node,
//   its LCA with the lower part, and that node must itself get one of the next k labels. With c
//   nodes, the new label hangs under a node or splits an edge (2c - 1 ways), becomes a pending
//   node, or hangs off a new pending node on an edge (c - 1 ways); a pending node whose last
//   label arrives must take it. Original LCAs are kept, and the shape never matters, only c and
//   the deadlines, so k = 0 is a product and k > 0 a DP over the k-bit mask of pending
//   deadlines, run from the last label back: O(m 2^k k) per test.
// original: corpus/submissions/5723059.cpp
// Bounds: tc_id: 0..=20; tcs: 1..=15
const P = 1000000007
type Z = Mod<P>
let tc_id: 0..=20 = read()
let tcs: 1..=15 = read()

for _ in 0..tcs:
    let n: 1..=30000 = read()
    let m: 0..=3000 = read()
    let k: 0..=10 = read()
    let parents: [n - 1]1..=n = read()
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

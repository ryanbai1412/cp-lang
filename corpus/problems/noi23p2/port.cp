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
//   the deadlines, so a DP over the Subset of pending deadlines, run from the last label
//   back, counts the ways: O(m 2^k k) per test (k = 0 leaves one state and a product).
// original: corpus/submissions/5723059.cpp
// Bounds: tc_id: 0..=20; tcs: 1..=15; n: 1..=30000; m: 0..=3000; k: 0..=10; parents: 1..=n
type Z = Mod<1_000_000_007>

let tc_id: 0..=20 = read()
let tcs: 1..=15 = read()

for _ in 0..tcs:
    let n: 1..=30000 = read()
    let m: 0..=3000 = read()
    let k: 0..=10 = read()
    let parents: [n - 1]1..=n = read()
    // a pending branch node in slot j must take a label within the next k - j
    domain Slot = 0..k
    var ways: [Slot.subsets]Z = Z(0)
    ways[Subset<Slot>.empty] = 1
    for i in (0..m).rev:
        var next: [Slot.subsets]Z = Z(0)
        for wait in Slot.subsets:
            let later = wait.shifted(1)
            if let last = Slot.last and wait.contains(last):
                next[wait] = ways[later]
                continue
            let c = n + i + wait.len
            var w = Z(2 * c - 1) * ways[later]
            if let first = Slot.first: w += Z(c - 1) * ways[later.with(first)]
            for b in wait: w += ways[wait.without(b).shifted(1)]
            next[wait] = w
        ways = next
    print(ways[Subset<Slot>.empty])

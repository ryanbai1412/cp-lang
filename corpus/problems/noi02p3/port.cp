// Source: NOI 2002 P3 (The Greedy Kuzuryu), https://dmoj.ca/problem/noi02p3
// limits: 1000 ms, 16 MB
// Problem: a tree of n <= 300 fruits, branch costs c <= 1e5, is split among m >= 2 heads, each
//   eating at least one fruit; the big head eats exactly k fruits, fruit 1 among them. A branch
//   whose two ends one head eats costs c. Print the minimum total cost, or -1 if impossible.
// Solution: with 2+ small heads they can alternate down the tree, so only big-big branches cost;
//   with one small head (m = 2) small-small branches cost too. Tree knapsack cost[v, who eats v,
//   fruits the big head eats in v's subtree], merging each child into its parent, children
//   first, over all count pairs: O(n^3). -1 when m - 1 + k > n.
// original: corpus/submissions/3718952.cpp
let (n, heads, k) = read<(1..=300, 2..=300, 1..=300)>()
domain F = 1..=n
let g = Graph.undirected(read<[n - 1](u: F, v: F, cost: 0..=100_000)>())
enum Head = Small | Big
fn clash(w: Head, wc: Head) -> Bool = w == wc and (w == Head.Big or heads == 2)
// eat(v)[w][x]: the cheapest for v's subtree when head w eats v and the big head eats x of its
// fruits.
fn eat(v: F, from: ?F) -> [Head]List<?Int>:
    var best: [Head]List<?Int> = [Head] w => if w == Head.Big: [None, Some(0)] else: [Some(0), None]
    for a in g[v]:
        if Some(a.to) == from: continue
        let sub = eat(a.to, Some(v))
        let size = best[Head.Big].len + sub[Head.Big].len - 1
        var merged: [Head]List<?Int> = [Head] _ => List.filled(size, None)
        for w in Head:
            for wc in Head:
                for (x, above) in best[w].items:
                    for (y, below) in sub[wc].items:
                        merged[w][x + y] min= above + below + (if clash(w, wc): a.cost else: 0)
        best = merged
    best
print(if heads - 1 + k > n: -1 else: eat(F.first!, None)[Head.Big][k]!)

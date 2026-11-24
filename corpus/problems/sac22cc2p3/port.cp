// Source: SAC '22 Code Challenge 2 P3 (Rating Choices), https://dmoj.ca/problem/sac22cc2p3
// limits: 1000 ms, 256 MB
// Problem: N <= 19 contests and rating R <= 1e9. A binary decision tree: each of the 2^N - 1
//   lines `u a b w` says that at node u you either take the contest (go to a, gain w) or skip
//   it (go to b). Starting at node 1, print the highest rating reachable at a leaf.
// Solution: tree DP from the leaves up: a leaf is worth 0, a fork the better of w + its take
//   child and its skip child. Print R + the root's value: O(2^N).
// original: corpus/submissions/4121049.cpp
// Bounds: k: 1..=19; r: 0..=1e9; forks.at, forks.take, forks.skip: 1..=2 ** (k + 1) - 1;
//   forks.gain: -1e9..=1e9
let (k, r): (1..=19, 0..=1e9) = read()
domain Fork = 0..2 ** k - 1
domain Node = 1..=2 ** (k + 1) - 1
let forks: [Fork](at: Node, take: Node, skip: Node, gain: -1e9..=1e9) = read()

var parent: [Node]?Node = None
var fork_at: [Node]?Fork = None
for f in Fork:
    let (at, take, skip, _) = forks[f]
    (parent[take], parent[skip], fork_at[at]) = (at, at, f)
let t = Rooted.from_parents(parent)

let best = t.up(
    max,
    lift: (x, c, p) => if let f = fork_at[p] and forks[f].take == c: x + forks[f].gain else: x,
    finish: (s, v) => if fork_at[v] == None: 0 else: s,
)
print(r + best[Node.first!])

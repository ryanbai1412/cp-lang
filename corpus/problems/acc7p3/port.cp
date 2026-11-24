// Source: Another Contest 7 P3 (Network Connections), https://dmoj.ca/problem/acc7p3
// Problem: n <= 1e5 users with distinct values f_i <= 1e9 (increasing with i) and m <= 1e5
//   existing friendships. Making i and j friends costs |f_i - f_j|; print the least cost that
//   connects everyone.
// Solution: sorted by value, a link between any two users can be replaced by the chain of
//   neighbouring pairs between them at the same total cost, so only the n - 1 neighbouring pairs
//   are candidates. Kruskal over the friendships at cost 0 and those pairs: O((n + m) log(n + m)).
// original: corpus/submissions/3540475.cpp
// Bounds: n: 1..=1e5; m: 0..=1e5; f: 0..=1e9; friends.u, friends.v: 1..=n

let (n, m): (1..=1e5, 0..=1e5) = read()
domain U = 1..=n
let f: [U]0..=1e9 = read()
let friends: [m](u: U, v: U) = read()

// existing friendships are free links; after them, only neighbours by value are worth linking
let by_value = U.sort(by: u => f[u])
let links = [(u:, v:, w: 0) for (u, v) in friends] + [(u:, v:, w: f[v] - f[u]) for (u, v) in by_value.adjacent]
print(kruskal(links).total)

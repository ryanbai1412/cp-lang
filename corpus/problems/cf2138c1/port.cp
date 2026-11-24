// Source: Codeforces 2138C1 (Maple and Tree Beauty, Easy Version),
//   https://codeforces.com/contest/2138/problem/C1
// limits: 3000 ms, 1024 MB
// Problem: a rooted tree of n <= 1000 vertices (parents p_i < i). Label exactly k vertices 0
//   and the rest 1 to maximise the longest common subsequence of all root-to-leaf label strings.
//   Multitest, t <= 50.
// Solution: beauty l is reachable iff depths 0..l-1 are each monochrome (l <= shallowest leaf
//   depth + 1) and some set of those levels plus some deeper vertices has exactly k vertices.
//   Shrinking l only frees vertices, so binary search l. The subset sum is a Bitset over level
//   sizes grouped by size (O(sqrt n) distinct), each group split into powers of two:
//   O(n sqrt(n) log n / 64) per check.
// Bounds: n: 2..=1000; k: 0..=1000; raw_par: 1..=n

@!multitest
let (n, k): (2..=1000, 0..=1000) = read()
domain V = 1..=n
domain S = 0..=n
let raw_par: [2..=n]V = read()
// parents come before their kids, so one sweep sets every depth
var depth: [V]Int = 0
var leaf: [V]Bool = true
for i in 2..=n:
    depth[V(i)] = depth[raw_par[i]] + 1
    leaf[raw_par[i]] = false
let leaf_depth = min(depth[v] for v in V if leaf[v])!
var freq: [0..n]Int = 0
for v in V: freq[depth[v]] += 1

fn is_good(l: Int) -> Bool:
    let deep = sum(freq[i] for i in l..n)
    S(k) in Bitset<S>.subset_sums([freq[i] for i in 0..l] + List.filled(deep, 1))

print((0..leaf_depth + 2).last_true(is_good)!)

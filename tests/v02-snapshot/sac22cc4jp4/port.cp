// Source: SAC '22 Code Challenge 4 Junior P4 (Obligatory Permutation Problem),
//   https://dmoj.ca/problem/sac22cc4jp4
// Problem: a permutation P of n <= 1e5. Start with A = P and K <= 1e18 times move every A[i] to
//   index P_i at once. Print A.
// Solution: after K moves A[P^K(i)] = P_i, i.e. A[P^(K-1)(j)] = j. Walk each cycle of P once and
//   write j at the position K - 1 steps further along its cycle. O(n).
// original: corpus/submissions/4416096.cpp
let n: 1..=1e5 = read()
let k: 1..=1e18 = read()
domain V = 1..=n

// a random shuffle, for stress tests
fn random_perm() -> [V]V:
    var a: [V]V = [V] v => v
    for v in V:
        let w = V(rand(1, n))
        if v != w: swap(mut a[v], mut a[w])
    a

let p: [V]V = read(gen: random_perm)

// j moves k - 1 places along its own cycle, so walk each cycle once and
// write the answers by position. No power of p is ever built.
var q: [V]V = [V] v => v
var seen: [V]Bool = false
for start in V:
    var cycle: List<V> = []
    var v = start
    while not seen[v]:
        seen[v] = true
        cycle.push(v)
        v = p[v]
    for t in 0..cycle.len: q[cycle[(t + k - 1) % cycle.len]] = cycle[t]
print(q)

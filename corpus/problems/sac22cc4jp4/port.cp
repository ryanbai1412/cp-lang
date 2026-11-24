// Source: SAC '22 Code Challenge 4 Junior P4 (Obligatory Permutation Problem),
//   https://dmoj.ca/problem/sac22cc4jp4
// limits: 1000 ms, 256 MB
// Problem: a permutation P of n <= 1e5. Start with A = P and K <= 1e18 times move every A[i] to
//   index P_i at once. Print A.
// Solution: after K moves A[P^K(i)] = P_i, i.e. A[P^(K-1)(j)] = j. Walk each cycle of P once and
//   write j at the position K - 1 steps further along its cycle. O(n).
// original: corpus/submissions/4416096.cpp

let n: 1..=1e5 = read()
let k: 1..=1e18 = read()
domain V = 1..=n
let p: Perm<V> = read()

var a: [V]V = [V] v => v
for cycle in Functional(p).cycles():
    for t in 0..cycle.len: a[cycle[(t + k - 1) % cycle.len]] = cycle[t]
print(a)

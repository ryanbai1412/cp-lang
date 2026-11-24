// Source: Mock CCC '23 Contest 1 J5 (Calculus Problem), https://dmoj.ca/problem/mccc5j5
// limits: 1000 ms, 512 MB
// Problem: N <= 1e6 problems worth d_0..d_{N-1} points and Q <= 5e5 people (m_j < N,
//   k_j <= 1e6); person j can solve problem i iff m_j xor i <= k_j. For each person print the
//   most points of a problem they can solve.
// Solution: the i with m xor i <= k are i = m xor k itself and, for each 1 bit d of k, the
//   aligned block of 2^d indices whose m xor i agrees with k above bit d and has a 0 at d.
//   Level d of a tower holds the max of each aligned 2^d block, so each block is one lookup.
//   O(N + Q log k).
// Bounds: n: 1..=1e6; q: 1..=5e5; points: 0..=2_147_483_647; m: 0..=n - 1; k: 0..=1e6
// original: corpus/submissions/5333196.cpp
let (n, q): (1..=1_000_000, 1..=500_000) = read()
domain Problem = 0..n
let points: [Problem]0..=2_147_483_647 = read()
var tower = [[points[p] for p in Problem]]
while tower.len <= 20:
    let below = tower.last!
    tower.push([max(below[i], below[min(i + 1, below.len - 1)]) for i in (0..below.len).step(2)])

fn most(m: Problem, k: 0..=1_000_000) -> Int:
    let x = Int(m) ^ k
    fn start(d: Int) -> Int = ((x >> (d + 1)) << (d + 1)) | (Int(m) & (1 << d))
    var best = if x < n: points[Problem(x)] else: 0
    for d in 0..=20:
        if k.bit(d) and start(d) < n: best max= tower[d][start(d) >> d]
    best

for _ in 0..q:
    let (m, k): (Problem, 0..=1_000_000) = read()
    print(most(m, k))

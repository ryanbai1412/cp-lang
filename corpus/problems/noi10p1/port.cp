// Source: NOI 2010 P1 (Energy Harvesting), https://dmoj.ca/problem/noi10p1
// Problem: a plant at every (x, y), 1 <= x <= n, 1 <= y <= m (n, m <= 1e5); the machine is
//   at (0, 0) and a plant with k plants on the segment to it loses 2k + 1 energy. Print
//   the total energy lost.
// Solution: (x, y) has gcd(x, y) - 1 plants on its segment, so the total is the sum of
//   pairs[g] * (2g - 1), pairs[g] the plants with gcd exactly g: (n / g)(m / g) minus
//   pairs[] of the larger multiples of g, filled from g = min(n, m) down. O(k log k).
// original: corpus/submissions/5670455.cpp
let n: 1..=100_000 = read()
let m: 1..=100_000 = read()
let k = min(n, m)

// pairs[g]: plants (x, y) with gcd(x, y) == g; a plant with gcd g has g - 1
// plants between it and the machine and loses 2 * (g - 1) + 1
var pairs: [1..=k]Int = 0
for g in (1..=k).rev:
    pairs[g] = (n / g) * (m / g) - sum(pairs[x] for x in (2 * g..=k).step(g))
print(sum(pairs[g] * (2 * g - 1) for g in 1..=k))

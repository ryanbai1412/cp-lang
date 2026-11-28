// Source: Codeforces 2211E (Minimum Path Cover), https://codeforces.com/contest/2211/problem/E
// limits: 4000 ms, 250 MB
// Problem: a tree on n <= 2e5 nodes rooted at 1, every child labelled above its parent, values
//   2 <= a_i <= 1e18. f(T) is the fewest vertical paths, each with gcd of values > 1, covering
//   every node once. Print f(S(n)), f(S(n-1)), ..., f(S(1)) online: node x (its value and
//   children) arrives only after f(S(x+1)) is printed. Interactive, multitest, sum of n <= 2e5.
// Solution: f(S(v)) is the children's f summed, plus 1 unless v can extend a child's top path.
//   good[v] is the lcm of the gcds the path through v can have in some optimal cover (it divides
//   a_v). v extends child u iff gcd(a_v, good[u]) > 1, so good[v] = lcm over u of
//   gcd(a_v, good[u]); when that is 1, v opens a new path: cost + 1 and good[v] = a_v.
//   O(n log A) for the gcds.
// Bounds: n: 1..=200_000; a: 1..=10 ** 18; k: 0..=n; children: 1..=n

let wire = Wire()
@!multitest
let n: 1..=200_000 = read()
domain V = 1..=n
var cost: [V]Int = 0
var good: [V]Int = 1
for cur in V.rev:
    let (a, k): (1..=10 ** 18, 0..=n) = read()
    let children: [0..k]V = read()
    for nxt in children:
        cost[cur] += cost[nxt]
        good[cur] = lcm(good[cur], gcd(a, good[nxt]))
    if good[cur] == 1:
        cost[cur] += 1
        good[cur] = a
    print(cost[cur])

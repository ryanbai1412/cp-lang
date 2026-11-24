// Source: AtCoder Educational DP T (Permutation), https://atcoder.jp/contests/dp/tasks/dp_t
// limits: 2000 ms, 1024 MB
// Problem: n <= 3000 and a string of n - 1 signs '<' / '>'. Count the permutations p of 1..=n with
//   p_i < p_{i+1} exactly where the i-th sign is '<', mod 1e9+7.
// Solution: build the permutation left to right by relative order: ways[r] counts orderings of
//   the first i values whose last one has r of them below it. The next value with r below it
//   follows a last value with fewer than r below ('<') or at least r ('>'), so each step is one
//   prefix (or suffix) sum: O(n^2).
// original: corpus/submissions/4131863.cpp
const P = 1000000007
type Z = Mod<P>
let n: 2..=3000 = read()
domain Rank = 0..n
domain Gap = 1..n
let signs: [Gap]Char = read()

// ways[r]: orderings of the values placed so far in which the last one has
// r of the others below it. A new value with r below it must follow a value
// that had fewer than r below it ('<'), or at least r ('>').
var ways: [Rank]Z = 0
ways[Rank.first!] = 1
for g in Gap:
    let placed = Rank(g.label)  // how many values are placed already
    var next: [Rank]Z = 0
    var run: Z = 0
    if signs[g] == '<':
        for r in ..=placed:
            next[r] = run
            run += ways[r]
    else:
        for r in (..placed).rev:
            run += ways[r]
            next[r] = run
    ways = next
print(ways.sum())

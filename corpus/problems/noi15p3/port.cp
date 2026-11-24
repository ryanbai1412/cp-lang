// Source: NOI 2015 P3 (Sushi Dinner), https://dmoj.ca/problem/noi15p3
// limits: 1000 ms, 512 MB
// Problem: count pairs of subsets X, Y of {2..n} with every x in X coprime to every y in Y,
//   n <= 500, modulo a runtime p <= 1e9.
// Solution: every number up to 500 has at most one prime above 19. Track which of the 8 small
//   primes each side already uses: dp[a, b], with a and b disjoint. Numbers sharing a big prime
//   must all avoid one side, so a group goes through the DP twice, once allowed only X (to_x)
//   and once only Y (to_y), and dp = to_x + to_y - dp removes the twice-counted "neither".
//   A number without a big prime is a group of its own. O(n 4^8).
// original: corpus/submissions/3782724.cpp

let (n, p): (2..=500, 1..=1e9) = read()
type Z = Mod<p>

domain Small = 0..8
let primes: [Small]Int = [2, 3, 5, 7, 11, 13, 17, 19]
type Mask = Subset<Small>
fn small(x: Int) -> Mask:
    var m = Mask.empty
    for i in Small:
        if x % primes[i] == 0: m = m.with(i)
    m

let sieve = Sieve(n)
fn big(x: Int) -> ?Int = sieve.factor(x).map(f => f.0).find(q => q > 19)
var groups: [2..=n]List<Int> = []
for x in 2..=n:
    if let q = big(x): groups[q].push(x)
    else: groups[x].push(x)

// one more number with small primes s, which may join X, or Y, or neither
fn join_x<T>(f: [Mask, Mask]T, s: Mask) -> [Mask, Mask]T:
    var g = f
    for a in Mask:
        for b in Mask:
            if s & b == Mask.empty: g[a | s, b] += f[a, b]
    g
fn join_y<T>(f: [Mask, Mask]T, s: Mask) -> [Mask, Mask]T:
    var g = f
    for a in Mask:
        for b in Mask:
            if s & a == Mask.empty: g[a, b | s] += f[a, b]
    g

var dp: [Mask, Mask]Z = 0
dp[Mask.empty, Mask.empty] = 1
for g in groups if not g.is_empty:
    var (to_x, to_y) = (dp, dp)
    for x in g:
        to_x = join_x(to_x, small(x))
        to_y = join_y(to_y, small(x))
    dp = [Mask, Mask] (a, b) => to_x[a, b] + to_y[a, b] - dp[a, b]
var total: Z = 0
for a in Mask:
    for b in Mask: total += dp[a, b]
print(total)

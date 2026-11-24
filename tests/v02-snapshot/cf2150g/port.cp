// Source: Codeforces 2150G (Counting Is Fun), https://codeforces.com/contest/2150/problem/G
// Problem: given x, y, k (k < x + y) and a binary string a of length x + y, count mod 998244353
//   the binary strings b > a (lexicographically) with exactly x zeros and y ones that split into a
//   prefix and a suffix both with longest non-decreasing subsequence (LNDS) >= k. Multitest; sum
//   of x and sum of y <= 5000.
// Solution: b is good iff the suffix after its shortest prefix with LNDS >= k has LNDS >= k. z
//   zeros and o ones give C(z + o, max(z, o, m)) strings with LNDS >= m, and LNDS(s + r) =
//   max(s.lnds + ones(r), s.zeros + LNDS(r)). Each b > a is a[..i] + '1' + rest at a zero a[i].
//   While a's prefix is below k, the states where s + rest first reaches LNDS k lie on one
//   anti-diagonal with ballot (reflection) counts, recorded as rays in a [Cnt, Cnt] grid and swept
//   once, O(k^2); once a's prefix reaches k, restart there and each flip adds a completion count.
@!multitest
use std.mod.{Comb, Mod}

type Z = Mod<998_244_353>

let (x, y, k): (1..=5000, 1..=5000, 1..=9999) = read()
let a: Str = read()
let comb = Comb<Z>(x + y)

struct Walk(zeros: Int, ones: Int, lnds: Int):
    fn push(self, c: Char) -> Walk =
        if c == '0': Walk(zeros + 1, ones, max(lnds, zeros + 1))
        else: Walk(zeros, ones + 1, lnds + 1)

enum Phase:
    Seeking
    Split(base: Walk)

fn long_lnds(z: Int, o: Int, m: Int) -> Z = comb.choose(z + o, max(z, o, m))
fn completions(s: Walk, z: Int, o: Int) -> Z =
    if o >= k - s.lnds: comb.choose(z + o, z) else: long_lnds(z, o, k - s.zeros)

domain Cnt = 0..=k
var rays: [Cnt, Cnt]Z = Z.zero
fn ray(z: Int, o: Int, w: Z): rays[Cnt(z), Cnt(o)] += w

fn first_hits_from(s: Walk):
    if s.lnds == k:
        ray(s.zeros, s.ones, Z.one)
        ray(s.zeros + 1, s.ones - 1, -Z.one)
        return
    let (e0, e1) = (k - s.zeros, k - s.lnds)
    for d in e1..e0 + e1:
        let top = comb.choose(d - 1, e1 - 1) - comb.choose(d - 1, e0)
        let main = comb.choose(d - 1, e0 - 1) - comb.choose(d - 1, e0)
        let (z, o) = (s.zeros + d - e1, s.ones + e1)
        ray(z, o, top)
        ray(z + 1, o - 1, main - top)

var ans = Z.zero
var walk = Walk(0, 0, 0)
var phase = Phase.Seeking
for c in a:
    if c == '0':
        let s = walk.push('1')
        match phase:
            Phase.Seeking => first_hits_from(s)
            Phase.Split(base) => ans += completions(
                s,
                x - base.zeros - s.zeros,
                y - base.ones - s.ones,
            )
    walk = walk.push(c)
    if let Phase.Seeking = phase and walk.lnds >= k:
        phase = Phase.Split(walk)
        walk = Walk(0, 0, 0)

for z in 0..k:
    for o in 1..=k:
        rays[Cnt(z + 1), Cnt(o - 1)] += rays[Cnt(z), Cnt(o)]
print(
    ans + sum(
        rays[z, o] * long_lnds(
            x - z.ord,
            y - o.ord,
            k,
        ) for z in Cnt for o in Cnt if rays[z, o] != Z.zero
    )
)

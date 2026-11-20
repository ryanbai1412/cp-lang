// NOI '18 P2 - Inverse (DMOJ: noi18p2, 25pt, TL 1.0s, ML 511MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// A permutation p of 1..n attains the bubble-sort lower bound 1/2 * sum |i - p_i|
// iff it avoids the pattern 321 (every non-prefix-max must be the smallest unused
// element). Equivalently, p is uniquely determined by its prefix-maximum sequence
// M_1 <= M_2 <= ... <= M_n = n with M_i >= i.
//
// For each position i where p matches q on 1..i-1 and chooses p_i > q_i:
// since min_unused <= q_i, p_i must be a new prefix max >= L = max(pref_max, q_i) + 1.
// By the reflection principle, the number of valid suffixes with M_i >= L is:
//   C(2n - i - L, n - i) - C(2n - i - L, n - L - 2)   (when L <= n, else 0).
@multitest
use std.mod.{Mod, Comb}

type Z = Mod<998244353>
// WISH MG1: static let Comb across @multitest with a lazy or max-bound table
static let comb = Comb<998244353>(1200005)

let n: 1..=6e5 = read()
domain Pos = 1..=n
domain Val = 1..=n
// WISH MG11: permutation input constraint `Perm<Val>` for cpl stress (not just `[Pos]Val`)
let q: [Pos]Val = read() where is_perm(q)

fn ballot(n: Int, i: Int, low: Int) -> Z:
    if low > n: return Z(0)
    let steps = 2 * n - i - low
    comb.choose(steps, n - i) - comb.choose(steps, n - low - 2)

var seen: [Val]Bool = false
var min_unused = 1
var pref_max = 0
var ans = Z(0)

for i in Pos:
    let qi = q[i].to_idx + 1
    let low = max(pref_max, qi) + 1
    ans += ballot(n, i.to_idx + 1, low)
    if qi > pref_max:
        pref_max = qi
    elif qi != min_unused:
        break
    seen[q[i]] = true
    while min_unused <= n and seen[Val(min_unused)]:
        min_unused += 1

print(ans)

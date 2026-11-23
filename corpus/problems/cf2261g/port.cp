// Source: Codeforces 2261G (1 2 3 and XOR, Where Did Everybody Go?),
//   https://codeforces.com/contest/2261/problem/G
// Problem: f(b) is the least b_1 xor ... xor b_k after adding any non-negative number to
//   one element of b. Given a_1..a_n (a_i < 2^25), print the sum of f over all subarrays.
//   1 <= n <= 4e5, t <= 1e4 tests, sum n <= 4e5.
// Solution: the increment turns b_i into any c >= b_i, and the xor becomes R ^ c with
//   R ^ b_i = X, the whole xor. Down X's bits from the top, c must copy b_i until the first X
//   bit b_i lacks (there c goes above b_i and then copies R), so f keeps the top X bits that
//   every element has, i.e. that the AND has. Each AND bit's X bit is the length's parity, so
//   f sums 2^b over the AND bits b with X == AND on all bits >= b. For a fixed right end
//   the suffix ANDs take <= 25 values; each group keeps a binary trie of the prefix xors of
//   its left ends, merged when two ANDs coincide. Walking a group's trie along pref ^ mask
//   counts, at each set bit b of mask, the left ends whose XOR agrees with the AND on bits
//   >= b; add 2^b times that. O(n * 25^2) plus trie merges.
use std.bitset.{BitTrie}

const L = 25
const FULL = (1 << L) - 1
type Word = 0..=FULL
struct Group:
    var mask: Word
    ends: BitTrie<L>

@!multitest
let n: 1..=4e5 = read()
let a: [0..n]Word = read()

// Narrow each suffix AND by `x`; neighbours whose ANDs become equal merge into one group.
fn shrink(mut groups: List<Group>, x: Word):
    for i in 0..groups.len: groups[i].mask &= x
    var i = 1
    while i < groups.len:
        if groups[i].mask != groups[i - 1].mask: i += 1
        else:
            groups[i - 1].ends.absorb(mut groups[i].ends)
            groups.remove(i)

// 2^b for each set bit b of `mask` and each left end whose xor first differs from it at b.
// The loop, not `sum(...)`: a generator under `sum` is collected into a list first (1.5x).
fn count_along(g: Group, pref: Word) -> Int:
    var total = 0
    with lint(loop_reduction: allow):
        for (bit, count) in g.ends.walk(pref ^ g.mask) if g.mask.bit(bit): total += count << bit
    total

var groups = List<Group>()
var pref: Word = 0
var ans = 0
for x in a:
    groups.push(Group(FULL, BitTrie<L>.of(pref)))
    shrink(mut groups, x)
    pref ^= x
    ans += sum(count_along(g, pref) for g in groups)
print(ans)

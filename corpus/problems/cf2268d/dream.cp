// Codeforces 2268 D: from i you may step left to any j < i (cost 1) or jump
// right to the next greater element up[i] (cost 1); sum all finite pairwise
// distances.
//
// - Pairs (a, b) with a > b have distance 1 -> base n * (n - 1) / 2.
// - Pairs with a < b have finite distance iff they share a prefix-max block
//   [l..=r], where r is the root of l's spine in the next-greater forest.
// - Each spine edge i -> u = up[i] on l -> .. -> r is charged for:
//   1. Cross-edge pairs a in [l..=i], b in (i..=r]: every such pair crosses
//      i -> u (+1), and b in (i, u) also steps left from u (+1 more).
//   2. Exiting (i, u) to b in (u..=r]: direct children of u other than i
//      (kids[u] - 1 vertices) reach u in 1 step; the rest of (i, u) take 2.
//   3. Pairs (a, b) with i < a < b <= u (total (u - i) * inside / 2):
//      * dist 1: a -> up[a] = b (one per a in (i, u), so `inside` pairs).
//      * dist 2: jump past b then step left (`up[a] - a - 1` per a in (i, u)),
//        or step into a child of b (`b - first[b] - kids[b]` per b in (i, u],
//        plus `kids[first[b]]` when b < u for paths through first[b]).
//      * dist 3: all remaining pairs in (i, u].
use std.seq.*

@!multitest
let n: 1..=5e5 = read()
domain I = 0..n
let p: [I](1..=n) = read()

let up = p.next_smaller(by: desc)
var first: [I]?I = None
var kids: [I]Int = 0
for i in I if let u = up[i]:
    if first[u] == None: first[u] = i
    kids[u] += 1

fn charge_edge(l: I, r: I, i: I, u: I) -> Int:
    let inside = (u - i) - 1
    let cross = (i - l + 1) * (r - i + inside)
    let exit_right = (r - u) * (2 * inside - (kids[u] - 1))
    let total_in = (u - i) * inside / 2
    var two = 0
    if let st = i.next:
        two = sum((up[a]! - a) - 1 for a in st..u)
        for b in st..=u if let fb = first[b]:
            two += (b - fb) - kids[b] + (if b < u: kids[fb] else: 0)
    let three = total_in - inside - two
    cross + exit_right + inside + 2 * two + 3 * three

var ans = n * (n - 1) / 2
var cur: ?I = I.first
while let l = cur:
    var r = l
    while let u = up[r]: r = u
    var i = l
    while let u = up[i]:
        ans += charge_edge(l, r, i, u)
        i = u
    cur = r.next
print(ans)

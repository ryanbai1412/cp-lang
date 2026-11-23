// Codeforces 2089 E.
//
// Vertices are labelled in preorder (each parent is the smaller endpoint) and
// are retired from n down to 2; after each retirement the answer is the
// constant term of the root's state. Every series is an EGF in the number of
// pending steps:
//   sub[v]     the subtree of v: f + integral(f) of its children's product;
//   before[v]  the product of the subtrees of v's earlier siblings;
//   dp[v]      the states parked at v, weighted by k! for k pending steps.
// Retiring t moves the states parked at t + 1 onto the path from t up to (not
// including) parent(t + 1), picking up the earlier siblings of each child on
// the way, then lifts every state on the path from t to the root, taking the
// siblings it climbs past back out of the carried sum. Because states carry
// k!, taking the steps of b out of a is the transposed product
// (a.transposed_mul(b))[i] = sum_j a[i + j] * b[j].
use std.ntt.{convolve}
use std.poly.{integral}

type Z = Mod<998_244_353>

fn add(a: List<Z>, b: List<Z>) -> List<Z>:
    var sum = a.resized(max(a.len, b.len), 0)
    for (i, x) in b.items: sum[i] += x
    sum

fn transposed_mul(a: List<Z>, b: List<Z>) -> List<Z>:
    if a.is_empty: return []
    convolve(a, b.rev)[b.len - 1..].to_list()

fn plus_integral(a: List<Z>) -> List<Z> = add(a, integral(a))

@!multitest
let n: 1..=250_000 = read()
domain V = 1..=n
var parent: [V]?V = none
repeat n - 1:
    let (u, v): (V, V) = read()
    parent[max(u, v)] = min(u, v)
let tree = Rooted.from_parents(parent)

var sub: [V]List<Z> = [1]
for v in V.rev:
    sub[v] = plus_integral(sub[v])
    if let p = parent[v]: sub[p] = convolve(sub[p], sub[v])

var before: [V]List<Z> = []
var siblings: [V]List<Z> = [1]
for v in V if let p = parent[v]:
    before[v] = siblings[p]
    siblings[p] = convolve(siblings[p], sub[v])

var dp: [V]List<Z> = []
var ans: List<Z> = [1]
for t in V.rev if t != V.first:
    let next = t.next
    let (moving, stop): (List<Z>, V) = match next:
        some(s) => (dp[s], parent[s]!)
        none => ([0, 1], V.first!)
    dp[t] = add(dp[t], moving)
    let chain = tree.path(t, stop)
    var ways: List<Z> = [1]
    for (prev, p) in chain[..chain.len - 1].with_prev:
        if let prev = prev: ways = convolve(ways, before[prev])
        dp[p] = add(dp[p], transposed_mul(moving, integral(ways)))
        ways = plus_integral(ways)
    if let s = next: dp[s] = []

    let up = tree.path(t, V.first!)
    var carry: List<Z> = []
    for (prev, p) in up.with_prev:
        if let prev = prev: carry = transposed_mul(carry, before[prev])
        let lifted: List<Z> = [0] + [x * (k + 1) for (k, x) in dp[p].items]
        (dp[p], carry) = (add(lifted, carry), add(carry, dp[p]))
    ans.push(dp[V.first!].first ?? 0)
print(ans)

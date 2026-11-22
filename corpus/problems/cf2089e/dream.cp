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
use std.poly.{Poly}

type Z = Mod<998_244_353>
type P = Poly<Z>

@!multitest
let n: Int = read()
domain V = 1..=n
var parent: [V]?V = none
repeat n - 1:
    let (u, v): (V, V) = read()
    parent[max(u, v)] = min(u, v)
let tree = Rooted.from_parents(parent)

var sub: [V]P = P.one
for v in V.rev:
    sub[v] += sub[v].integral()
    if let p = parent[v]: sub[p] *= sub[v]

var before: [V]P = P.one
var siblings: [V]P = P.one
for v in V if let p = parent[v]:
    before[v] = siblings[p]
    siblings[p] *= sub[v]

var dp: [V]P = P.zero
var ans: List<Z> = [1]
for t in V.rev if t != V.first:
    let moving = if let s = t.next: dp.take(s) else: P.x
    let stop = t.next.map(s => parent[s]!) ?? V.first
    dp[t] += moving
    var ways = P.one
    for (c, p) in tree.path(t, stop).drop_last().with_prev:
        if let c = c: ways *= before[c]
        dp[p] += moving.transposed_mul(ways.integral())
        ways += ways.integral()

    var carry = P.zero
    for (c, p) in tree.path(t, V.first).with_prev:
        if let c = c: carry = carry.transposed_mul(before[c])
        (dp[p], carry) = (P.x * (P.x * dp[p]).deriv() + carry, carry + dp[p])
    ans.push(dp[V.first!][0])
print(ans)

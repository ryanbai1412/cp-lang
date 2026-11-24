// Source: IOI 2022 Day 2 P1, Digital Circuit, https://dmoj.ca/problem/ioi22p4
// Problem: a tree of N threshold gates and M source gates (N, M <= 1e5, P[i] < i); a gate with
//   c inputs takes a parameter p in 1..=c and is 1 iff at least p inputs are 1. Q <= 1e5 times,
//   toggle sources L..=R and count parameter assignments making gate 0 output 1, mod 1000002022.
// Solution: by linearity each gate passes a count through once, so the answer is the sum over
//   lit sources of the product of all[] (assignments in a subtree) over the subtrees hanging
//   off the source's root path. Compute all[] bottom-up and worth[] top-down with prefix and
//   suffix products (the modulus is not prime). A lazy segment tree over sources keeps
//   (lit worth, total worth) under range flips. O(N + M + Q log M).
// original: corpus/submissions/4783516.cpp (grader functions init / count_ways)
// stdin: n m q, the n + m parents (-1 for gate 0), the m source states, then
// q lines l r. stdout: the count after each toggle.
// Bounds: n, m: 1..=100_000; q: 0..=100_000; parent: -1..=n - 1; lit: 0..=1
use std.lazy.{flip_action}

type Z = Mod<1_000_002_022>
let (n, m, q): (1..=100_000, 1..=100_000, 0..=100_000) = read()
domain Node = 0..n + m
domain Source = n..n + m
let parent: [Node]-1..n = read()
let lit: [Source]0..=1 = read()
assert parent[Node.first!] == -1 and all(0 <= parent[v] < Int(v) for v in Node.first!.after)

var kids: [Node]List<Node> = []
for v in Node.first!.after: kids[Node(parent[v])].push(v)
let gates = (Node.first!..Node(n)).rev

// ways[u]: parameter assignments inside u's subtree; a gate with c inputs has c choices
var ways: [Node]Z = Z(1)
for u in gates: ways[u] = Z(kids[u].len) * prod(ways[v] for v in kids[u])

// a lit source is worth the assignments of everything off its path to the root: each gate on
// the path passes the count through once. The modulus is not prime, so "all children but one"
// is a prefix times a suffix product
var worth: [Node]Z = Z(1)
for u in gates.rev:
    var after = [Z(1)]
    for v in kids[u].rev: after.push(after.last! * ways[v])
    var before = worth[u]
    for (i, v) in kids[u].items:
        worth[v] = before * after[kids[u].len - 1 - i]
        before *= ways[v]

// per source (worth if lit, worth if dark); a toggle swaps the two over a range
fn join(a: Flip<Z>, b: Flip<Z>) -> Flip<Z> = Flip(a.now + b.now, a.flipped + b.flipped)
var sources = SegTree<Source, Flip<Z>, Bool>(Monoid(Flip(Z(0), Z(0)), join), [Source] s => Flip.of(lit[s] == 1, worth[Node(Int(s))], Z(0)), act: flip_action<Z>(+))
for _ in 0..q:
    let (l, r): (Source, Source) = read()
    sources.apply(l..=r, true)
    print(sources.all.now)

// Source: Codeforces 2135F (To the Infinity), https://codeforces.com/contest/2135/problem/F
// limits: 2000 ms, 1024 MB
// Problem: a full binary tree on n <= 4e5 nodes (n odd), root 1; a leaf has f(x) = x and an inner
//   node f_u = f_l ^ f_r. Print the nodes sorted by f_u as x -> infinity, ties by smaller index.
//   Multitest, sum n <= 4e5.
// Solution: f_u = x ^ (product of f over the right children down u's left spine), so a node is the
//   multiset of those right children, compared largest first. With gen = max(gen l, gen r + 1), a
//   value only holds lower generations and beats every lower-generation value, so rank generation
//   by generation; a node's key is where its run of equal values starts in the output. A multiset
//   is a persistent segment tree over keys (set[u] = set[l] + key[r]) with random-weighted hashes;
//   compare by descending to the largest key whose subtrees hash apart, O(log n). Sorting each
//   generation: O(n log^2 n).

use std.marks.{Arena, Id}
use std.rand.{Rng}

const MAX_N = 400_000
struct Node(l: Id<Node>, r: Id<Node>, cnt: 0..=MAX_N, h: U64)

@!multitest
let n: 1..=MAX_N = read()
domain V = 1..=n
domain Gen = 0..n
domain Key = 0..n
let kids: [V]?(l: V, r: V) = read<[V](0..=n, 0..=n)>().map(
    (l, r) => if l == 0: None else: Some((V(l), V(r)))
)

var by_gen: [Gen]List<V> = []
@droppable
fn place(v: V) -> Gen:
    let g = if let (l, r) = kids[v]: max(place(l), Gen(place(r) + 1)) else: Gen.first!
    by_gen[g].push(v)
    g
place(V.first!)

var rng = Rng.new(2135)
let weight: [Key]U64 = [Key] _ => rng.next()
var pool = Arena<Node>.new(cap: 20 * n)
let empty = pool.alloc(Node(Id(0), Id(0), 0, U64(0)))

fn insert(t: Id<Node>, span: Range<Key>, k: Key) -> Id<Node>:
    let at = pool[t]
    var (l, r) = (at.l, at.r)
    if let (left, right) = span.halve():
        if left.contains(k): l = insert(at.l, left, k) else: r = insert(at.r, right, k)
    pool.alloc(Node(l, r, at.cnt + 1, at.h + weight[k]))

fn less(a0: Id<Node>, b0: Id<Node>) -> Bool:
    var (a, b, span) = (pool[a0], pool[b0], Key.all)
    while a.h != b.h and a.cnt > 0 and b.cnt > 0:
        let (left, right) = span.halve() else: break
        if pool[a.r].h != pool[b.r].h:
            (a, b, span) = (pool[a.r], pool[b.r], right)
        else:
            (a, b, span) = (pool[a.l], pool[b.l], left)
    a.cnt < b.cnt

var set: [V]Id<Node> = empty
var key: [V]Key = Key.first!
var order: List<V> = []
for vs in by_gen:
    for v in vs:
        if let (l, r) = kids[v]: set[v] = insert(set[l], Key.all, key[r])
    let ranked = vs.sorted(
        cmp: (a, b) => if pool[set[a]].h == pool[set[b]].h: a < b else: less(set[a], set[b])
    )
    for run in ranked.runs(by: v => pool[set[v]].h):
        for v in run: key[v] = Key(order.len)
        order.extend(run)
print(order)

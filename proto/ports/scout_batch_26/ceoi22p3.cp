// CEOI '22 P3 - Prize (25pt, interactive): two rooted trees on the same n
// vertices with hidden positive edge lengths. Pick k vertices, then ask k - 1
// questions "? a b"; after "!" each is answered with four numbers: the
// distances from a and from b to their lowest common ancestor, in tree 1
// and in tree 2. Then t pairs of picked vertices arrive and the distance of
// each pair in both trees must be printed.
// Pick the first k vertices of a preorder of tree 1 (closed under parents)
// and ask about neighbours in tree 2's preorder. Those neighbours' common
// ancestors are exactly the branching points of the auxiliary tree of the
// picked set in tree 2, and each answer ties two of its vertices by a
// known depth difference; in tree 1 the same answers tie picked vertices
// to picked ancestors. Both sets of ties are connected, so one pass over
// them fixes every depth up to a shift, and a distance is
// depth a + depth b - 2 * depth lca.
// original: corpus/submissions/4728198.cpp
use std.tree.{Rooted}

let (n, k, q, t): (2..=1e6, 2..=1e5, Int, 1..=1e5) = read()
domain V = 1..=n
domain Pick = 0..k
domain Ask = 0..(k - 1)

// WISH TR3: `let boss: [V]?V(none: -1) = read()` and `Rooted<V>.from_parents(boss)`
fn read_tree():
    let boss: [V]Int = read()
    var kids: [V]List<(to: V)> = [V] _ => []
    var root = V.first
    for v in V:
        if boss[v] == -1: root = v
        else: kids[V(boss[v])].push((to: v))
    Rooted(u => kids[u], root)

let one = read_tree()
let two = read_tree()

let one_pre = [v for v in one.at]
var picked: List<V> = [one_pre[i.ord] for i in Pick]
// WISH TR15: `print(picked)` should write the list on one line; the interpreter
// shows `[1, 2]`, so the picks go out one per line (the judge reads tokens).
for v in picked: print(v)
picked.sort(by: v => two.tin[v].ord)
for i in Ask: print("?", picked[i.ord], picked[i.ord + 1])
// WISH TR16: an `interactor`/`ask` block would own the flush; the interpreter
// has no `flush()`, and check.py runs it unbuffered instead.
print("!")

// ties[x]: (y, d) with depth[y] = depth[x] + d
fn depths<Pre>(r: Rooted<V, Pre>, col: Int, replies: [Ask](Int, Int, Int, Int)) -> [V]Int:
    var ties: [V]List<(to: V, d: Int)> = [V] _ => []
    for i in Ask:
        let (a, b) = (picked[i.ord], picked[i.ord + 1])
        let top = r.lca(a, b)
        let (da, db) = if col == 1: (replies[i].0, replies[i].1) else: (replies[i].2, replies[i].3)
        ties[top].push((to: a, d: da))
        ties[a].push((to: top, d: -da))
        ties[top].push((to: b, d: db))
        ties[b].push((to: top, d: -db))
    var depth: [V]Int = 0
    var known: [V]Bool = false
    var stack: List<V> = [picked[0]]
    known[picked[0]] = true
    // WISH TR17: `while let x = stack.pop():` (the cheat sheet's `pop() -> ?T`);
    // the interpreter's pop traps on an empty list.
    while stack.len > 0:
        let x = stack.pop()
        for e in ties[x]:
            if known[e.to]: continue
            known[e.to] = true
            depth[e.to] = depth[x] + e.d
            stack.push(e.to)
    depth

let replies: [Ask](Int, Int, Int, Int) = read()
let d1 = depths(one, 1, replies)
let d2 = depths(two, 2, replies)
domain Query = 0..t
let pairs: [Query](a: V, b: V) = read()
for p in pairs:
    print(d1[p.a] + d1[p.b] - 2 * d1[one.lca(p.a, p.b)], d2[p.a] + d2[p.b] - 2 * d2[two.lca(p.a, p.b)])

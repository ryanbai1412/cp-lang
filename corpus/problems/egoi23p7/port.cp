// Source: EGOI 2023 Day 2 (Sopsug), https://dmoj.ca/problem/egoi23p7
// limits: 5000 ms, 1024 MB
// Problem: N <= 3e5 buildings, M <= 3e5 existing directed tubes (u sends garbage to v) that
//   must be used and K <= 3e5 ordered pairs where no tube may be built. Print N - 1 tubes that
//   form a tree directed toward one root, including the existing ones, or "NO".
// Solution: the required tubes must form a forest (Rooted.try_from_parents, else NO); each
//   tree is a block that hangs under another block. Block c may not hang under block k when
//   every building of k is a banned parent of c's root (tallied by root). Search the
//   complement of that "may not" graph, scanning only unreached blocks so each kept one is
//   paid for by a ban; search from the last start to get one root, then again from it.
//   NO unless every block hangs; each link takes the first allowed building of k's tree.
//   O((N + M + K) log N).
// original: corpus/submissions/5678057.cpp
// Bounds: n: 1..=3e5; m1, m2: 0..=3e5; need.child, need.parent, e.child, e.parent: 0..n
let (n, m1, m2) = read<(1..=3e5, 0..=3e5, 0..=3e5)>()
domain V = 0..n
let need = read<[m1](child: V, parent: V)>()

var up: [V]?V = None
for e in need:
    if up[e.child] != None:
        print("NO")
        return
    up[e.child] = Some(e.parent)
let forest = Rooted.try_from_parents(up) else: print("NO"); return
var bans: [V]Set<V> = []
var banned_by: [V]List<V> = []
for e in read<[m2](child: V, parent: V)>():
    if e.parent not in bans[e.child]:
        bans[e.child].insert(e.parent)
        banned_by[e.child].push(e.parent)
let blocks = forest.roots.collect()
var cannot: [V]List<V> = []
for c in blocks:
    for (k, hits) in banned_by[c].tally(by: v => forest.root_of(v)).items():
        if hits == forest.size(k): cannot[k].push(c)

// The block each reached block hangs below, searching from `starts` in turn, and the last start.
fn search(starts: List<V>) -> (root: V, below: [V]?V):
    var below: [V]?V = None
    var seen: [V]Bool = false
    var mark: [V]?V = None
    var left = blocks.clone()
    var root = starts.first!
    for s in starts:
        if seen[s]: continue
        (root, seen[s]) = (s, true)
        var todo = [s]
        while let k = todo.pop():
            for c in cannot[k]: mark[c] = Some(k)
            var rest: List<V> = []
            for c in left:
                if seen[c]: continue
                if mark[c] == Some(k):
                    rest.push(c)
                else:
                    (seen[c], below[c]) = (true, Some(k))
                    todo.push(c)
            left = rest
    (root:, below:)

let top = search(blocks).root
let hang = search([top]).below
if not blocks.all(c => c == top or hang[c] != None):
    print("NO")
else:
    for e in need: print(e.child, e.parent)
    for c in blocks:
        if let k = hang[c]: print(c, forest.subtree(k).find(v => v not in bans[c])!)

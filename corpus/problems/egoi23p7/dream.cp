// EGOI 2023 P7 Find the Tree: build a rooted spanning tree that uses every required arc and no
// banned arc, or print NO. The required arcs must form a forest; each of its trees is a block
// that hangs under another block. Block c may hang under block k unless every vertex of k is a
// banned parent of c's root. The blocks then need a spanning tree in the complement of the
// "may not" graph: a search that scans only the blocks it has not reached, each kept block paid
// for by a ban. O((N + M + K) log N).
let (n, m1, m2) = read<(1..=3e5, 0..=3e5, 0..=3e5)>()
domain V = 0..n
let need = read<[m1](child: V, parent: V)>()
var bans: [V]Set<V> = []
var banned_by: [V]List<V> = []
for e in read<[m2](child: V, parent: V)>():
    if e.parent not in bans[e.child]:
        bans[e.child].insert(e.parent)
        banned_by[e.child].push(e.parent)

var up: [V]?V = None
for e in need:
    if up[e.child] != None:
        print("NO")
        return
    up[e.child] = Some(e.parent)
let forest = Rooted.try_from_parents(up) else: print("NO"); return
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
    var left = blocks
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

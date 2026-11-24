// NOIP 2018 P3, tuned: port.cp pairing each node's sorted chains by binary search, skipping
// paired ones through next-free links, instead of popping a `SortedBag` built per node; each
// node reads its children's chains from one table, and one with at most one chain builds nothing.
let (n, m): (2..=5e4, 1..=5e4) = read()
domain Junction = 1..=n
domain Road = 1..n
let roads: [Road](u: Junction, v: Junction, len: 1..=1e4) = read()
let g = Graph.undirected(roads)
let tree = Rooted(u => g.out(u), Junction.first!)

var up_len: [Junction]Int = 0  // the road from v to its parent
for (u, v, len) in roads:
    if tree.parent(u) == v: up_len[u] = len else: up_len[v] = len

// Pair chains (all shorter than k) into tracks: the shortest chain takes
// the shortest partner that completes it. Returns the pairs made and the
// longest chain left alone.
fn pair_up(chains: List<Int>, k: Int) -> (tracks: Int, spare: Int):
    let xs = chains.sorted()
    let d = xs.len
    var free = [i for i in 0..=d]  // free[i] leads to the first unpaired chain at or after i
    fn first_free(i: Int) -> Int:
        var root = i
        while free[root] != root: root = free[root]
        var at = i
        while free[at] != root:
            let next = free[at]
            free[at] = root
            at = next
        root
    var (tracks, spare) = (0, 0)
    for i in 0..d:
        if free[i] != i: continue
        free[i] = i + 1
        let j = first_free((i + 1..d).first_true(j => xs[j] + xs[i] >= k) ?? d)
        if j < d:
            free[j] = j + 1
            tracks += 1
        else: spare = xs[i]
    (tracks:, spare:)

fn tracks_of(k: Int) -> Int:
    var tracks = 0
    var up: [Junction]Int = 0  // the chain v passes to its parent; 0 when none (roads are >= 1)
    for v in tree.at.rev:
        var (open, last) = (0, 0)
        for c in tree.children(v):
            if up[c] > 0: (open, last) = (open + 1, up[c])
        let made = if open <= 1: (tracks: 0, spare: last)
            else: pair_up([up[c] for c in tree.children(v) if up[c] > 0], k)
        tracks += made.tracks
        if tree.parent(v) != None:
            let chain = made.spare + up_len[v]
            if chain >= k: tracks += 1 else: up[v] = chain
    tracks

print((1..=sum(r.len for r in roads)).last_true(k => tracks_of(k) >= m)!)

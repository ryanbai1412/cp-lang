// Source: NOIP 2018 Senior P3 (Constructing Tracks), https://dmoj.ca/problem/noip18p3
// limits: 1000 ms, 512 MB
// Problem: a tree of n <= 5e4 junctions with road lengths <= 1e4. Build m <= n - 1 tracks,
//   each a path, no road in two tracks; print the largest possible length of the shortest.
// Solution: binary search the length k. For one k, go up the tree: each junction gets the
//   chains ending at its children (plus their up-road; a chain of length >= k is a track),
//   pairs them in a SortedBag (the shortest unpaired takes the shortest partner that reaches
//   k) and passes the longest unpaired chain to its parent. O(n log n log(sum of lengths)).
// original: corpus/submissions/4002963.cpp
// Bounds: n: 2..=5e4; m: 1..=5e4; roads.u, roads.v: 1..=n; roads.len: 1..=1e4
let (n, m): (2..=5e4, 1..=5e4) = read()
domain Junction = 1..=n
domain Road = 1..n
let roads: [Road](u: Junction, v: Junction, len: 1..=1e4) = read()
let g = Graph.undirected(roads)
let tree = Rooted(u => g.out(u), Junction.first!)

var up_len: [Junction]Int = 0  // the road from v to its parent
for (u, v, len) in roads:
    if tree.parent(u) == v: up_len[u] = len else: up_len[v] = len

// For one k, walk up the tree. Each junction pairs the chains its children hand it (all
// shorter than k) into tracks: the shortest chain takes the shortest partner that completes
// it. The longest chain left alone, plus the road up, goes to the parent. One bag serves
// every junction, since pairing empties it.
fn tracks_of(k: Int) -> Int:
    var tracks = 0
    var up: [Junction]Int = 0  // the chain v hands its parent, 0 for none
    var bag = SortedBag<Int>()
    for v in tree.at.rev:
        for c in tree.children(v):
            if up[c] > 0: bag.add(up[c])
        var spare = 0
        while let x = bag.pop_first():
            if bag.take_first(y => x + y >= k) != None: tracks += 1 else: spare = x
        if tree.parent(v) != None:
            let chain = spare + up_len[v]
            if chain >= k: tracks += 1 else: up[v] = chain
    tracks

// the longest k that still gives m tracks
print((1..=sum(r.len for r in roads)).last_true(k => tracks_of(k) >= m)!)

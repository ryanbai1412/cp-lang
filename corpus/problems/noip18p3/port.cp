// Source: NOIP 2018 Senior P3 (Constructing Tracks), https://dmoj.ca/problem/noip18p3
// Problem: a tree of n <= 5e4 junctions with road lengths <= 1e4. Build m <= n - 1 tracks,
//   each a path, no road in two tracks; print the largest possible length of the shortest.
// Solution: binary search the length k. For one k, go up the tree: each junction gets the
//   chains ending at its children (plus their up-road; a chain of length >= k is a track),
//   greedily pairs them into tracks (the shortest unpaired takes the shortest unused partner
//   that reaches k, found by binary search, used ones skipped by path compression) and
//   passes the longest unpaired chain to its parent. O(n log n log(sum of lengths)).

let (n, m): (Int, Int) = read()
domain Junction = 1..=n
domain Road = 1..n
let roads: [Road](u: Junction, v: Junction, len: Int) = read()
let g = Graph<Junction, Road>.undirected(roads)
let tree = Rooted(u => g.out(u), Junction.first!)

var up_len: [Junction]Int = 0  // the road from v to its parent
for (u, v, len) in roads:
    if tree.parent(u) == v: up_len[u] = len else: up_len[v] = len

// Pair chains (all shorter than k) into tracks: the shortest chain takes
// the shortest partner that completes it. Returns the pairs made and the
// longest chain left alone.
fn pair_up(chains: List<Int>, k: Int) -> (tracks: Int, spare: Int):
    let a = chains.sorted()
    var skip = [i for i in 0..=a.len]  // skip[i]: the first unused chain at or after i
    fn unused(i: Int) -> Int:
        var j = i
        while skip[j] != j:
            skip[j] = skip[skip[j]]
            j = skip[j]
        j
    var (tracks, spare) = (0, 0)
    for i in 0..a.len:
        if unused(i) != i: continue
        skip[i] = i + 1
        let lo = (i + 1..a.len).first_true(mid => a[mid] >= k - a[i]) ?? a.len  // the first chain after i that is at least k - a[i]
        let j = unused(lo)
        if j < a.len:
            skip[j] = j + 1
            tracks += 1
        else:
            spare = a[i]
    (tracks:, spare:)

fn tracks_of(k: Int) -> Int:
    var tracks = 0
    var chains = [Junction] v => List<Int>()
    for v in tree.at.rev:
        let made = pair_up(chains[v], k)
        tracks += made.tracks
        if let p = tree.parent(v):
            let chain = made.spare + up_len[v]
            if chain >= k: tracks += 1 else: chains[p].push(chain)
    tracks

// the longest k that still gives m tracks
print((1..=sum(r.len for r in roads)).last_true(k => tracks_of(k) >= m) ?? 0)

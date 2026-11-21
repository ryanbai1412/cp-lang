// NOIP 2018 Track Construction: a tree of roads with lengths. Build m
// tracks, each a path, no road used twice, and make the shortest track as
// long as possible.
//
// Binary search the length k. For one k, go up the tree: every junction
// gets the chains that end at its children, pairs them into tracks of
// length at least k, and passes the longest chain left over to its parent.

let (n, m): (Int, Int) = read()
domain Junction = 1..=n
domain Road = 1..n
let roads: [Road](u: Junction, v: Junction, len: Int) = read()
let g = Graph<Junction, Road>.undirected(roads)
let tree = Rooted(u => g.out(u), Junction.first)
let bottom_up = tree.preorder.rev

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
        var (lo, hi) = (i + 1, a.len)  // the first chain after i that is at least k - a[i]
        while lo < hi:
            let mid = (lo + hi) / 2
            if a[mid] >= k - a[i]: hi = mid else: lo = mid + 1
        let j = unused(lo)
        if j < a.len:
            skip[j] = j + 1
            tracks += 1
        else:
            spare = a[i]
    (tracks: tracks, spare: spare)

fn tracks_of(k: Int) -> Int:
    var tracks = 0
    var chains = [Junction] v => List<Int>()
    for v in bottom_up:
        let made = pair_up(chains[v], k)
        tracks += made.tracks
        if let p = tree.parent(v):
            let chain = made.spare + up_len[v]
            if chain >= k: tracks += 1 else: chains[p].push(chain)
    tracks

var (lo, hi) = (0, sum(r.len for r in roads))  // the longest k that still gives m tracks
while lo < hi:
    let mid = (lo + hi + 1) / 2
    if tracks_of(mid) >= m: lo = mid else: hi = mid - 1
print(lo)

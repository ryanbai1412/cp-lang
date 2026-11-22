// noip18p3 Track Construction: build m edge-disjoint paths in a weighted tree, making the shortest
// as long as possible. Binary search the length k. Going up, each vertex pairs the chains its
// children hand it into tracks of length at least k (the shortest chain takes the shortest
// partner that completes it) and passes the longest unpaired chain to its parent.
let (n, m) = read<(2..=50_000, 1..=50_000)>()
domain V = 1..=n
let g = Graph.undirected(read<[n - 1](u: V, v: V, w: 1..=10_000)>())
let t = Rooted(g, root: V.first!)
fn tracks(k: Int) -> Int:
    var made = 0
    fn up(v: V) -> Int:
        var chains = SortedBag<Int>()
        for c in t.children(v):
            let chain = up(c) + t.w(c)
            if chain >= k:
                made += 1
            else:
                chains.add(chain)
        var left = 0
        while chains.pop_first() is some(x):
            if chains.first_true(y => x + y >= k) is some(y):
                chains.remove(y)
                made += 1
            else:
                left = x
        left
    up(t.root)
    made
print((1..=500_000_000).last_true(k => tracks(k) >= m)!)

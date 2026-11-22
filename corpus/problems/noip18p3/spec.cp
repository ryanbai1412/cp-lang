// noip18p3 Track Construction: build m edge-disjoint paths in a weighted tree, making the shortest
// as long as possible. Binary search the length k. Going up, each vertex pairs the chains its
// children hand it into tracks of length at least k (the shortest chain takes the shortest
// partner that completes it) and passes the longest unpaired chain to its parent.
let (n, m) = read<(2..=5e4, 1..=5e4)>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V, w: 1..=1e4)>()), root: V.first!)

fn tracks(k: Int) -> Int:
    var made = 0
    fn up(v: V) -> Int:
        var chains = SortedBag.of(List(t.child_arcs(v).map(a => up(a.to) + a.w)))
        made += chains.drain_range(k..).count()
        var spare = 0
        while let x = chains.pop_first():
            if chains.take_first(y => x + y >= k) is Some(_):
                made += 1
            else:
                spare = x
        spare
    up(t.root)
    made

print((1..=5e8).last_true(k => tracks(k) >= m)!)

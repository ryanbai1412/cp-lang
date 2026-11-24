// NOIP 2018 Constructing Tracks: binary search the shortest track length k. For one k, each
// vertex pairs the chains its children hand up, the shortest with the shortest partner that
// completes it, and hands the longest leftover up to its parent.
let (n, m): (2..=5e4, 1..=5e4) = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V, len: 1..=1e4)>()), root: V.first!)

fn tracks(k: Int) -> Int:
    var made = 0
    fn up(v: V) -> Int:
        let ups = [up(a.to) + a.len for a in t.child_arcs(v)]
        made += ups.count(c => c >= k)
        var chains = SortedBag.of([c for c in ups if c < k])
        var spare = 0
        while let x = chains.pop_first():
            if chains.take_first(y => x + y >= k) is Some(_): made += 1 else: spare = x
        spare
    let _ = up(t.root)
    made

print((1..=5e8).last_true(k => tracks(k) >= m)!)

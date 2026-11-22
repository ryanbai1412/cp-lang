let (n, m) = read<(2..=5e4, 1..=5e4)>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V, w: 1..=1e4)>()), root: V.first!)

fn tracks(k: Int) -> Int:
    var made = 0
    fn up(v: V) -> Int:
        var chains = SortedBag(t.down(v).map(a => up(a.to) + a.w))
        made += chains.drain(k..).count()
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

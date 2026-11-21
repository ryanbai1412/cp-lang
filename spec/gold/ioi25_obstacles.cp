// Going down to a warmer row only widens what is reachable, so only the rows
// where the temperature reaches a new maximum matter. At such a row r the free
// columns form maximal runs with h < t[r]; each run is a layer. Every earlier
// layer lies inside one run of the new row, and the llama can drop from it into
// that run through any of its columns drier than every row in between; the first
// and last such columns are the drop. Layers and drops form a forest. With
// s ≤ d, (0, s) and (0, d) connect inside l..=r iff their row-0 layers have a
// common ancestor, every drop on s's path has its last column ≥ l, and every
// drop on d's path has its first column ≤ r.
@grader
mod obstacles:
    domain Row, Col                                    // sized by initialize
    struct Layer(row: Row, cols: Range<Col>)
    struct Drop(first: Col, last: Col)

    // A path's binding drops: the rightmost `first` and the leftmost `last`.
    fn tightest(a: Drop, b: Drop) -> Drop = Drop(max(a.first, b.first), min(a.last, b.last))

    var layers: Lifted<Layer, Drop>                    // forest + binary lifting: lca, path fold
    var home: [Col]?Node                               // row-0 layer of each free column

    fn initialize(t: [Row]0..=1e9, h: [Col]0..=1e9):
        let coldest = RangeMin(t)
        let driest = RangeMin(h)
        let wettest = RangeMax(h)
        var forest = Forest<Layer, Drop>()
        var open = SortedMap<Col, Node>()              // layers not yet inside a later run
        var waiting = Queue(Col.sorted(by: h))         // columns not yet free, driest first
        home = none
        for r in t.record_highs():                     // rows with a new maximum temperature
            let runs = waiting.pop_while(c => h[c] < t[r])
                .map(c => wettest.span(around: c, while: x => x < t[r]))
                .distinct()
            for run in runs:
                let layer = forest.add(Layer(r, run))
                for (_, child) in open.drain(in: run):
                    let warm = coldest[forest[child].row..=r]
                    let cols = forest[child].cols
                    if let first = driest.first(in: cols, where: x => x < warm):
                        let last = driest.last(in: cols, where: x => x < warm)!
                        forest.link(child, parent: layer, Drop(first, last))
                open[run.start] = layer
                if r == Row.first: home[run] = layer
        layers = forest.lift(tightest, identity: Drop(Col.first, Col.last))

    fn can_reach(l: Col, r: Col, s: Col, d: Col) -> Bool:
        let (s, d) = minmax(s, d)
        let (a, b) = (home[s]!, home[d]!)
        let top = layers.lca(a, b) ?? return false
        layers.fold(a, up_to: top).last >= l and layers.fold(b, up_to: top).first <= r

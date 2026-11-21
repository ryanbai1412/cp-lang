// Going down to a warmer row only widens what is reachable, so only the rows
// where the temperature reaches a new maximum matter. At such a row r the free
// columns form maximal runs with h < t[r]; each run is a layer. Every earlier
// layer lies inside one run of the new row, and the llama can drop from it into
// that run through any of its columns drier than every row in between; the first
// and last such columns are the drop. Layers and drops form a forest. With
// s ≤ d, (0, s) and (0, d) connect inside l..=r iff their row-0 layers have a
// common ancestor, every drop on s's path has its last column ≥ l, and every
// drop on d's path has its first column ≤ r.
use std.tree.{Lift}

@grader
mod obstacles:
    domain Row                                         // both sized by initialize
    domain Col
    struct Layer(row: Row, cols: Range<Col>)
    struct Drop(first: Col, last: Col)

    // A path's binding drops: the rightmost `first` and the leftmost `last`.
    fn tightest(a: Drop, b: Drop) -> Drop = Drop(max(a.first, b.first), min(a.last, b.last))

    var layers: Lift<Col, Drop>                        // a layer is named by the column that opened it
    var home: [Col]?Col                                // row-0 layer of each free column

    fn initialize(t: [Row]0..=1e9, h: [Col]0..=1e9):
        let coldest = SparseTable(min, t)
        let driest = SparseTable(min, h)
        let wettest = SparseTable(max, h)
        // The maximal run of columns around `c` drier than `limit`.
        fn run_around(c: Col, limit: Int) -> Range<Col>:
            let lo = (Col.first!..=c).bsearch(l => wettest.fold(l..=c) < limit)
            let hi = (c..=Col.last!).bsearch(r => wettest.fold(c..=r) < limit, last: true)
            lo..hi
        var layer: [Col]?Layer = none
        var up: [Col]?(Col, Drop) = none
        var open = SortedMap<Col, Col>()               // layers not yet inside a later run
        var waiting = Deque.of(Col.range.sorted(by: c => h[c]))   // columns not yet free, driest first
        home = none
        for r in t.record_highs():                     // rows with a new maximum temperature
            let opened = waiting.pop_front_while(c => h[c] < t[r])
            for (id, run) in [(c, run_around(c, t[r])) for c in opened].distinct(by: p => p.1):
                layer[id] = Layer(r, run)
                for (_, child) in open.drain(after: run.l.l, while: (c, _) => c in run):
                    let below = layer[child]!
                    let warm = coldest.fold(below.row..=r)
                    let cols = below.cols
                    if let first = cols.first_true(c => driest.fold(cols.l..c.r) < warm):
                        let last = cols.last_true(c => driest.fold(c.l..cols.r) < warm)!
                        up[child] = (id, Drop(first, last))
                open[run.first!] = id
                if r == Row.first!:
                    for c in run: home[c] = id
        layers = Lift(up, tightest, identity: Drop(Col.first!, Col.last!))

    fn can_reach(l: Col, r: Col, s: Col, d: Col) -> Bool:
        let (s, d) = minmax(s, d)
        let (a, b) = (home[s]!, home[d]!)
        let top = layers.lca(a, b) ?? return false
        layers.fold(a, up_to: top).last >= l and layers.fold(b, up_to: top).first <= r

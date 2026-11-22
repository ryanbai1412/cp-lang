// IOI 2019 "Rectangles": count rectangles whose every interior cell is
// strictly lower than the border cells in its row and column.
pub fn count_rectangles(a: List<List<Int>>) -> Int:
    domain R = [a.len]
    domain C = [a[0].len]
    let h = [R, C] (r, c) => a[r][c]

    // a bounded span of a line: a nonempty interior below both of its ends
    let row_spans = [R] r => Set(h.row(r).bounded_spans())
    let col_spans = [C] c => Set(h.col(c).bounded_spans())

    // how far each span repeats: rows downward, columns rightward
    var depth = Map<(R, Range<C>), Int>()
    for r in R.rev:
        for s in row_spans[r]:
            depth[(r, s)] = 1 + (r.next.and_then(d => depth.get((d, s))) ?? 0)
    var breadth = Map<(C, Range<R>), Int>()
    for c in C.rev:
        for s in col_spans[c]:
            breadth[(c, s)] = 1 + (c.next.and_then(d => breadth.get((d, s))) ?? 0)

    let across = row_spans.items.flat_map((r, ss) => ss.map(s => ((r, s.l), (wide: s.len, deep: depth[(r, s)])))).group()
    let down = col_spans.items.flat_map((c, ss) => ss.map(s => ((s.l, c), (tall: s.len, broad: breadth[(c, s)])))).group()

    // a corner pairs one span of each kind when each reaches across the other
    (R, C).sum(of: corner => dominance_pairs(
        across.get(corner) ?? [],
        down.get(corner) ?? [],
        fits: (x, y) => x.deep >= y.tall and y.broad >= x.wide,
    ))

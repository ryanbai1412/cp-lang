// IOI 2019 "Rectangles": count rectangles whose every interior cell is
// strictly lower than the border cells in its row and column. An interior is
// a valley of each of its rows and of each of its columns. From the interior's
// top-left cell (r, c), a row valley starting at c reaches `deep` rows down and a
// column valley starting at r reaches `broad` columns right; a pair forms a
// rectangle when each reaches across the other, a 2D dominance count per cell.
@grader
fn count_rectangles(a: List<List<Int>>) -> Int:
    domain R = [a.len]
    domain C = [a[0].len]
    let h = [R, C] (r, c) => a[Int(r)][Int(c)]

    var deep = Map<(R, Range<C>), Int>()
    var across = Map<(R, C), List<(wide: Int, deep: Int)>>()
    for r in R.rev:
        for s in h[r].valleys():
            let d = 1 + (r.next.and_then(below => deep.get((below, s))) ?? 0)
            deep[(r, s)] = d
            across.get_or_insert((r, s.l.r!)).push((wide: s.len, deep: d))
    var broad = Map<(C, Range<R>), Int>()
    var down = Map<(R, C), List<(tall: Int, broad: Int)>>()
    for c in C.rev:
        for s in h[.., c].valleys():
            let b = 1 + (c.next.and_then(right => broad.get((right, s))) ?? 0)
            broad[(c, s)] = b
            down.get_or_insert((s.l.r!, c)).push((tall: s.len, broad: b))

    domain Width = 1..=C.len
    var widths = Fenwick<Width, Int>()
    fn fitting(xs: List<(wide: Int, deep: Int)>, ys: List<(tall: Int, broad: Int)>) -> Int:
        let xs = xs.sorted(by: x => desc(x.deep))
        var taken = 0
        var count = 0
        for y in ys.sorted(by: y => desc(y.tall)):
            while taken < xs.len and xs[taken].deep >= y.tall:
                widths.add(Width(xs[taken].wide), 1)
                taken += 1
            count += widths.sum(..=Width(min(y.broad, C.len)))
        for x in xs.take(taken):
            widths.add(Width(x.wide), -1)
        count

    across.items.sum(of: ((corner, xs)) => fitting(xs, down.get(corner) ?? List()))

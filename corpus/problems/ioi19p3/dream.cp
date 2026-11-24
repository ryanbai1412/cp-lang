// IOI 2019 "Rectangles": count rectangles whose every interior cell is
// strictly lower than the border cells in its row and column.
//
// An interior is a valley of each of its rows and each of its columns. A row
// valley starting at a cell reaches `deep` rows down while the same span stays
// a valley; a column valley reaches `broad` columns right. A row valley and a
// column valley from the same top-left cell form a rectangle when each reaches
// across the other: a 2D dominance count per cell.
let n = read<1..=2500>()
let m = read<1..=2500>()
domain R = [n]
domain C = [m]
let h = read<[R, C]0..=7_000_000>()

// how far each valley repeats: row spans downward, column spans rightward
var across: [R, C]List<(wide: Int, deep: Int)> = [R, C] (r, c) => List()
var below = Map<Range<C>, Int>()
for r in R.rev:
    let here = Map(([C] c => h[r, c]).valleys().map(s => (s, 1 + (below.get(s) ?? 0))))
    for (s, deep) in here.items: across[r, s.first!].push((wide: s.len, deep:))
    below = here
var down: [R, C]List<(tall: Int, broad: Int)> = [R, C] (r, c) => List()
var right = Map<Range<R>, Int>()
for c in C.rev:
    let here = Map(([R] r => h[r, c]).valleys().map(s => (s, 1 + (right.get(s) ?? 0))))
    for (s, broad) in here.items: down[s.first!, c].push((tall: s.len, broad:))
    right = here

// a row valley fits a column valley when it is deep enough and narrow enough
domain Width = 1..=C.count
var widths = Fenwick<Width, Int>()
fn fitting(xs: List<(wide: Int, deep: Int)>, ys: List<(tall: Int, broad: Int)>) -> Int:
    let xs = xs.sorted(by: x => desc(x.deep))
    var taken = 0
    var count = 0
    for y in ys.sorted(by: y => desc(y.tall)):
        while taken < xs.len and xs[taken].deep >= y.tall:
            widths.add(Width(xs[taken].wide), 1)
            taken += 1
        count += widths.fold(..=Width(y.broad))
    for x in xs.take(taken): widths.add(Width(x.wide), -1)
    count

print(R.sum(of: r => C.sum(of: c => fitting(across[r, c], down[r, c]))))

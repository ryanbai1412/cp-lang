// IOI 2019 "Rectangles": count rectangles whose every interior cell is
// strictly lower than the border cells in its row and column. An interior is
// a valley of each of its rows and of each of its columns. From the interior's
// top-left cell, a row valley starting there reaches `deep` rows down and a
// column valley starting there reaches `broad` columns right; a pair forms a
// rectangle when each reaches across the other, a 2D dominance count per cell.
// Reads the sample grader's input (`n m`, then the grid); see frictions.
let n = read<1..=2500>()
let m = read<1..=2500>()
domain R = [n]
domain C = [m]
let h = read<[R, C]0..=7_000_000>()

var across: [R, C]List<(wide: Int, deep: Int)> = [R, C] (r, c) => List()
var below = Map<(C, C), Int>()  // valleys keyed by their ends
for r in R.rev:
    var here = Map<(C, C), Int>()
    for s in ([C] c => h[r, c]).valleys():
        let ends = (s.first!, s.last!)
        here[ends] = 1 + (below.get(ends) ?? 0)
        across[r, s.first!].push((wide: s.len, deep: here[ends]))
    below = here
var down: [R, C]List<(tall: Int, broad: Int)> = [R, C] (r, c) => List()
var right = Map<(R, R), Int>()
for c in C.rev:
    var here = Map<(R, R), Int>()
    for s in ([R] r => h[r, c]).valleys():
        let ends = (s.first!, s.last!)
        here[ends] = 1 + (right.get(ends) ?? 0)
        down[s.first!, c].push((tall: s.len, broad: here[ends]))
    right = here

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
    for x in xs.take(taken):
        widths.add(Width(x.wide), -1)
    count

print(R.sum(of: r => C.sum(of: c => fitting(across[r, c], down[r, c]))))

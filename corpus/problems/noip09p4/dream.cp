// NOIP 2009 Target Sudoku: complete the sudoku maximising Σ value × ring weight (10 at the
// centre down to 6 at the border), or print -1. Backtrack on the empty cell with fewest options.
domain D = [9]
let grid = read [D, D]0..=9
let weight = (r: D, c: D) => 10 - max(abs(Int(r) - 4), abs(Int(c) - 4))
var used = (row: [D]Subset<1..=9>, col: [D]Subset<1..=9>, box: [D]Subset<1..=9>)
fn box_of(r: D, c: D) -> D = D(Int(r) / 3 * 3 + Int(c) / 3)
fn options(r: D, c: D) = (1..=9).all - used.row[r] - used.col[c] - used.box[box_of(r, c)]

var empty: List<(D, D)> = []
var base = 0
for ((r, c), v) in grid.items:
    if v == 0: empty.push((r, c)); continue
    if v in used.row[r] or v in used.col[c] or v in used.box[box_of(r, c)]: print(-1); return
    for s in [mut used.row[r], mut used.col[c], mut used.box[box_of(r, c)]]: s.add(v)
    base += v * weight(r, c)

fn search(left: List<(D, D)>) -> ?Int:
    let Some(cell) = left.min_by(cell => options(cell).len) else: return some(0)
    let (r, c) = cell
    var best: ?Int = none
    for v in options(r, c).items:
        place(r, c, v)
        best max= search(left.without(cell)).map(s => s + v * weight(r, c))
        unplace(r, c, v)
    best
print(search(empty).map(s => s + base) ?? -1)

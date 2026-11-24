// NOIP 2009 Target Sudoku: complete the sudoku maximising Σ value × ring weight (10 at the
// centre down to 6 at the border), or print -1. Backtrack on the empty cell with fewest options.
domain Line = 0..9
domain Digit = 1..=9
type Digits = Subset<Digit>
var grid: [Line, Line]0..=9 = read()
let weight = (r: Line, c: Line) => 10 - max(abs(r.idx - 4), abs(c.idx - 4))
fn box_of(r: Line, c: Line) -> Line = Line(r.idx / 3 * 3 + c.idx / 3)
var row: [Line]Digits = Digits.empty  // the digits placed in each row, column and box
var col: [Line]Digits = Digits.empty
var box: [Line]Digits = Digits.empty
fn options(r: Line, c: Line) -> Digits = Digits.full - (row[r] | col[c] | box[box_of(r, c)])

fn mark(r: Line, c: Line, d: Digit, on: Bool):
    row[r].set(d, on)
    col[c].set(d, on)
    box[box_of(r, c)].set(d, on)
    grid[r, c] = if on: d.label else: 0

// the given digits' score, or None when two of them clash
fn givens() -> ?Int:
    var score = 0
    for (r, c) in product(Line.all, Line.all):
        if grid[r, c] == 0: continue
        let d = Digit(grid[r, c])
        if d not in options(r, c): return None
        mark(r, c, d, true)
        score += d.label * weight(r, c)
    Some(score)

// the best score the empty cells can add, or None when they can't be filled
fn search() -> ?Int:
    let empty = product(Line.all, Line.all).filter(((r, c)) => grid[r, c] == 0)
    let Some((r, c)) = empty.min(by: ((r, c)) => options(r, c).len) else: return Some(0)
    var best: ?Int = None
    for d in options(r, c).items:
        mark(r, c, d, true)
        if let s = search(): best max= s + d.label * weight(r, c)
        mark(r, c, d, false)
    best

print(givens().and_then(g => search().map(s => g + s)) ?? -1)

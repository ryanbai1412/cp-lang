// noip09p4 Target Sudoku: finish the grid maximizing the sum of digit times cell worth (10 at the
// centre, 6 on the rim), or -1 when there is none (clashing givens included). Backtrack, always
// branching on the blank with the fewest digits.
domain Line = 0..9
domain Cell = (Line, Line)
domain Digit = 1..=9
let given = read<[Line, Line]0..=9>()
var board: [Cell]?Digit = [Cell] p => Digit.at(given[p])
var in_row: [Line]Subset<Digit> = Subset<Digit>.empty
var in_col: [Line]Subset<Digit> = Subset<Digit>.empty
domain Band = 0..3
var in_box: [Band, Band]Subset<Digit> = Subset<Digit>.empty
fn box(p: Cell) -> (Band, Band) = (Band(Int(p.0) / 3), Band(Int(p.1) / 3))
fn worth(p: Cell) -> Int = 10 - max(abs(Int(p.0) - 4), abs(Int(p.1) - 4))
fn toggle(p: Cell, d: Digit):
    let ((r, c), b, bit) = (p, box(p), Subset<Digit>.single(d))
    in_row[r] ^= bit
    in_col[c] ^= bit
    in_box[b] ^= bit
fn free(p: Cell) -> Subset<Digit> = Subset<Digit>.full - (in_row[p.0] | in_col[p.1] | in_box[box(p)])
var best: ?Int = None
fn fill(score: Int):
    let p = Cell.filter(p => board[p] == None).min(by: p => free(p).len) else:
        best max= score
        return
    for d in free(p):
        toggle(p, d)
        board[p] = d
        fill(score + worth(p) * Int(d))
        board[p] = None
        toggle(p, d)
fn givens() -> ?Int:
    var score = 0
    for p in Cell:
        if let d = board[p]:
            if d not in free(p): return None
            toggle(p, d)
            score += worth(p) * Int(d)
    score
if let score = givens(): fill(score)
print(best ?? -1)

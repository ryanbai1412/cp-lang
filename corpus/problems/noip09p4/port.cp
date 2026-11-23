// NOIP 2009 P4, Target Sudoku: finish a sudoku so that the sum of every
// digit times the worth of its cell is as large as possible. Cells are
// worth 10 in the centre down to 6 on the border. Print -1 if it cannot
// be finished.
// original: corpus/submissions/5185258.cpp
domain Line = 0..9
domain Box = 0..9
domain Digit = 1..=9
var board: [Line, Line]0..=9 = read()  // WISH L20: [Line, Line]?Digit, read with 0 for none

type Digits = Subset<Digit>
var in_row: [Line]Digits = Digits.empty  // the digits already placed in each row, column and box
var in_col: [Line]Digits = Digits.empty
var in_box: [Box]Digits = Digits.empty

fn box_of(r: Line, c: Line) -> Box: Box(r.idx / 3 * 3 + c.idx / 3)  // WISH L12: (r, c).block(3, 3)
fn worth(r: Line, c: Line) -> Int: 10 - max(abs(r.idx - 4), abs(c.idx - 4))

fn fits(r: Line, c: Line, d: Digit) -> Bool: d not in in_row[r] | in_col[c] | in_box[box_of(r, c)]

fn mark(r: Line, c: Line, d: Digit, on: Bool):  // WISH L11: in_row[r].toggle(d), or in_row[r] ^= d
    let b = box_of(r, c)
    in_row[r] = if on: in_row[r].with(d) else: in_row[r].without(d)
    in_col[c] = if on: in_col[c].with(d) else: in_col[c].without(d)
    in_box[b] = if on: in_box[b].with(d) else: in_box[b].without(d)

var best: ?Int = none
fn fill(score: Int):
    // The blank with the fewest digits left is the cheapest to branch on.
    var pick: ?(Line, Line) = none  // WISH L13: let (r, c) = min_by(blanks(), p => options(p)) else: ...
    var fewest = 10
    for r in Line:
        for c in Line:
            if board[r, c] == 0:
                let options = count(fits(r, c, d) for d in Digit)
                if options < fewest:
                    fewest = options
                    pick = (r, c)
    let (r, c) = pick else:
        best max= score
        return
    for d in Digit:
        if fits(r, c, d):  // WISH L14: with mark(r, c, d): fill(...) undoes itself
            mark(r, c, d, true)
            board[r, c] = d.label
            fill(score + d.label * worth(r, c))
            board[r, c] = 0
            mark(r, c, d, false)

fn given() -> ?Int:
    var score = 0
    for r in Line:
        for c in Line:
            if board[r, c] != 0:
                let d = Digit(board[r, c])
                if not fits(r, c, d): return none
                mark(r, c, d, true)
                score += d.label * worth(r, c)
    return score

if let score = given(): fill(score)
print(best ?? -1)

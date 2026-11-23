// CCC 1999 S5 Letter Arithmetic: n puzzles, each three words of capital
// letters, the first two adding up to the third (SEND + MORE = MONEY).
// Different letters are different digits and no number starts with 0. Each
// puzzle has one answer: print the three numbers, one to a line, and a
// blank line after each puzzle.
//
// Add as on paper: column by column from the right, giving a digit to each
// new letter of the two top rows and checking the bottom row against the
// sum, so a wrong guess fails in the column where it is made.
// original: corpus/submissions/5515588.cpp
domain Letter = 'A'..='Z'
domain Digit = 0..10
domain Row = 0..3

let puzzles: Int = read()
for _ in 0..puzzles:
    let read_rows: [Row]Str = read()
    let rows = [read_rows[r] for r in Row]  // as a list: `fill` steps through rows by number
    let width = max(row.len for row in rows)
    var digit: [Letter]?Int = None
    var used: [Digit]Bool = false
    var leads: [Letter]Bool = false
    for row in rows: leads[Letter(row[0])] = true

    /// The letter of a row in a column, columns counted from the right.
    fn at(row: Int, col: Int) -> ?Letter:
        let word = rows[row]
        if col < word.len: Some(Letter(word[word.len - 1 - col])) else: None

    /// Give `x` the digit `d` if that is allowed, and go on with `rest`; take it back on failure.
    fn give(x: Letter, d: Int, rest: () -> Bool) -> Bool:
        if let old = digit[x]: return old == d and rest()
        if used[Digit(d)] or (d == 0 and leads[x]): return false
        (digit[x], used[Digit(d)]) = (Some(d), true)
        if rest(): return true
        (digit[x], used[Digit(d)]) = (None, false)
        false

    /// Fill in from `row` of `col` on; `total` is the column's sum so far, carry included.
    fn fill(col: Int, row: Int, total: Int) -> Bool:
        if col == width: return total == 0
        if row == 2:
            if let x = at(2, col): return give(x, total % 10, () => fill(col + 1, 0, total / 10))
            return total % 10 == 0 and fill(col + 1, 0, total / 10)
        if let x = at(row, col):
            if let d = digit[x]: return fill(col, row + 1, total + d)
            for d in Digit:
                if give(x, d.ord, () => fill(col, row + 1, total + d.ord)): return true
            return false
        fill(col, row + 1, total)

    fill(0, 0, 0)
    for row in rows: print(Str([Char(Int('0') + digit[Letter(c)]!) for c in row]))
    print()

// Source: CCC 1999 S5 (Letter Arithmetic), https://dmoj.ca/problem/ccc99s5
// Problem: n puzzles, each three words of at most 20 capital letters with word1 + word2 =
//   word3 (SEND + MORE = MONEY). Give distinct letters distinct digits, no leading zero; each
//   puzzle has exactly one answer. Print the three numbers, one per line, and a blank line
//   after each puzzle.
// Solution: backtracking that adds as on paper: column by column from the right, give a digit
//   to each new letter of the two top rows, then force the bottom row's letter to the column
//   sum mod 10 and carry the rest. A wrong guess fails in the column where it is made, so the
//   search prunes far below 10! assignments.
// original: corpus/submissions/5515588.cpp
domain Letter = 'A'..='Z'
domain Digit = 0..10
domain Row = 0..3

let puzzles: Int = read()
for _ in 0..puzzles:
    let read_rows: [Row]Str = read()
    let rows = [read_rows[r] for r in Row]  // as a list: `fill` steps through rows by number
    let width = [row.len for row in rows].max()!
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

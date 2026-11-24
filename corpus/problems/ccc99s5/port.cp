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

type At = (col: Int, row: Row, total: Int)  // a column (from the right), a row, the column's sum so far

@!multitest
let words: [Row]Str = read()
let width = max(w.len for w in words)
var digit: [Letter]?Int = None
var used: [Digit]Bool = false
var leads: [Letter]Bool = false
for w in words: leads[Letter(w[0])] = true

fn letter(row: Row, col: Int) -> ?Letter:
    if col < words[row].len: Some(Letter(words[row][words[row].len - 1 - col])) else: None

/// Fill in from `p` on, first giving the letter `x` the digit `d` when asked to; every digit
/// given is taken back on failure.
fn fill(p: At, give: ?(x: Letter, d: Int) = None) -> Bool:
    if let (x, d) = give:
        if let old = digit[x]: return old == d and fill(p)
        if used[Digit(d)] or (d == 0 and leads[x]): return false
        (digit[x], used[Digit(d)]) = (Some(d), true)
        if fill(p): return true
        (digit[x], used[Digit(d)]) = (None, false)
        return false
    let (col, row, total) = p
    if col == width: return total == 0
    if row == Row.last!:
        let next: At = (col: col + 1, row: Row.first!, total: total / 10)
        if let x = letter(row, col): return fill(next, give: Some((x, total % 10)))
        return total % 10 == 0 and fill(next)
    let below = Row(row.ord + 1)
    if let x = letter(row, col):
        if let d = digit[x]: return fill((col:, row: below, total: total + d))
        return Digit.any(d => fill((col:, row: below, total: total + d.ord), give: Some((x, d.ord))))
    fill((col:, row: below, total:))

assert fill((col: 0, row: Row.first!, total: 0))  // every puzzle has an answer
for w in words: print(Str([Char(Int('0') + digit[Letter(c)]!) for c in w]))
print()

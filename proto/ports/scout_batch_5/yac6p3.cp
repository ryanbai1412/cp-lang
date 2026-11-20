// Yet Another Contest 6 P3 (interactive): two cells of an n x m grid are
// hidden. "? x y" returns the walking distance from (x, y) to the nearer
// one. Print "! x1 y1 x2 y2" after at most 5 questions.
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
domain X = 1..=n
domain Y = 1..=m
type Cell = (x: X, y: Y)

fn near(c: Cell) -> Int = ask("?", c.x, c.y, limit: 5)              // WISH K21

// From a corner the distance fixes one diagonal of the nearer hidden cell:
// x + y from two opposite corners, x - y from the other two.
let sum_low = near((X.first, Y.first)) + 2
let sum_high = n + m - near((X.last, Y.last))
let diff_right = n - 1 - near((X.last, Y.first))
let diff_left = near((X.first, Y.last)) - m + 1

fn cell(sum: Int, diff: Int) -> ?Cell:
    if (sum + diff) % 2 != 0: return none
    (X.of((sum + diff) / 2)?, Y.of((sum - diff) / 2)?)              // WISH K16: a checked conversion

// Either the two corners with y = 1 see the same cell, or the two with x = 1 do.
if let p = cell(sum_low, diff_right) and near(p) == 0:
    print("!", p, cell(sum_high, diff_left)!)                       // WISH G4-like: a record prints its fields
else:
    print("!", cell(sum_low, diff_left)!, cell(sum_high, diff_right)!)

// Source: Yet Another Contest 6 P3 (No More Cell Phone Messaging),
//   https://dmoj.ca/problem/yac6p3
// Problem: interactive, non-adaptive: two distinct hidden cells (A, B), (C, D) of an N x M grid
//   (N, M <= 1e9). Query "? X Y" returns min(|X - A| + |Y - B|, |X - C| + |Y - D|); after at
//   most 5 queries print "! A B C D" (either order).
// Solution: the four corners give min(x + y), min(y - x), min(x - y) and max(x + y) over the
//   two cells. Either the min-sum cell also has the min y - x (the other then has the max sum
//   and the min x - y) or the pairing is crossed; a fifth query at the first pairing's
//   min-sum cell, when it is a grid cell, tells which. Each cell follows from its sum and
//   difference. O(1).
// original: corpus/submissions/5149597.cpp; judge: interactor.py
// Bounds: n, m: 1..=1e9
let n: 1..=1e9 = read()
let m: 1..=1e9 = read()
domain X = 1..=n
domain Y = 1..=m
type Cell = (X, Y)

fn near(c: Cell) -> Int:
    let d: 0..=2e9 = ask("?", c.0, c.1)
    d

// the grid cell with x + y = sum and x - y = diff, if there is one
fn cell(sum: Int, diff: Int) -> ?Cell:
    if (sum + diff) % 2 != 0: return None
    if let x = X.at((sum + diff) / 2), let y = Y.at((sum - diff) / 2): Some((x, y)) else: None

let low_sum = near((X.first!, Y.first!)) + 2
let low_y_minus_x = near((X.last!, Y.first!)) - n + 1
let low_x_minus_y = near((X.first!, Y.last!)) - m + 1
let high_sum = n + m - near((X.last!, Y.last!))

let (p, q) =
    if let first = cell(low_sum, -low_y_minus_x), near(first) == 0:
        (first, cell(high_sum, low_x_minus_y)!)
    else:
        (cell(low_sum, low_x_minus_y)!, cell(high_sum, -low_y_minus_x)!)
print("!", p.0, p.1, q.0, q.1)

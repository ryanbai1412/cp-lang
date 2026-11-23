// Source: CCO 2018 P4 (Gradient Descent), https://dmoj.ca/problem/cco18p4
// Problem: interactive. N <= 100 hidden tokens on an R x C grid (R, C <= 1e7); a cell's score
//   is the sum of Manhattan distances to all tokens. Ask "? p q" for a score at most K times
//   (K <= 170, 75 in the last subtask), then print "! Z", the minimum score.
// Solution: the score is a convex function of the row plus one of the column, so the best row
//   does not depend on the column: take the first row where stepping down column 1 stops
//   decreasing the score, then the first such column along that row. Queries are cached in a
//   HashMap, but each `find` scans in order, so it asks up to R + C questions, not O(log).
// As it builds today: the cache is a map keyed by one packed number, and each axis is a
// search on the slope (K1, K5, K7).
// original: corpus/submissions/4801866.cpp; judge: cco18p4_judge.py
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=100 = read()
domain Row = 1..=rows
domain Col = 1..=cols

var seen: HashMap<Int, Int> = []
fn cost(r: Row, c: Col) -> Int:
    if let known = seen.get(r.label * (cols + 1) + c.label): return known
    let total: 0..=4e9 = ask("?", r, c)
    seen[r.label * (cols + 1) + c.label] = total
    total

fn row_turns(r: Row) -> Bool:
    if let s = Row.at(r + 1): return cost(s, Col.first!) >= cost(r, Col.first!)
    true
let r = Row.find(r => row_turns(r))!
fn col_turns(c: Col) -> Bool:
    if let d = Col.at(c + 1): return cost(r, d) >= cost(r, c)
    true
let c = Col.find(c => col_turns(c))!
let best = cost(r, c)
print("!", best)

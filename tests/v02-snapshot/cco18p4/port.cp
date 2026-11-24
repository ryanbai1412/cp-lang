// Source: CCO 2018 P4 (Gradient Descent), https://dmoj.ca/problem/cco18p4
// Problem: interactive. N <= 100 hidden tokens on an R x C grid (R, C <= 1e7); a cell's score
//   is the sum of Manhattan distances to all tokens (<= 2e9). Ask "? p q" for a score at most K
//   times (K <= 170, 75 in the last subtask), then print "! Z", the minimum score.
// Solution: the score is a convex function of the row plus one of the column, so the best row
//   does not depend on the column: golden-section search (ternary_min) down column 1 for the
//   best row, then along that row for the best column. Probes are reused and scores cached,
//   so it asks about 2 log_phi 1e7 + 6 = 73 questions.
// original: corpus/submissions/4801866.cpp
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=170 = read()
domain Row = 1..=rows
domain Col = 1..=cols

// A memo may not do I/O (spec 6.4), so the asked scores are cached by hand.
var seen: HashMap<(Row, Col), Int> = []
fn score(r: Row, c: Col) -> Int:
    if let known = seen.get((r, c)): return known
    let s: 0..=2e9 = ask("?", r, c)
    seen[(r, c)] = s
    s

let r = ternary_min(Row.first!..=Row.last!, i => score(i, Col.first!))!
let c = ternary_min(Col.first!..=Col.last!, j => score(r, j))!
print("!", score(r, c))

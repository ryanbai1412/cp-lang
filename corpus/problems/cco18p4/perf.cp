// Tuned: the accepted C++'s ternary search: a tie between the two probes narrows to the span
// between them, which std's golden section does not, so the 1e7 x 1e7 max case asks 50
// questions instead of 68.
//
// Source: CCO 2018 P4 (Gradient Descent), https://dmoj.ca/problem/cco18p4
// Problem: interactive. N <= 100 hidden tokens on an R x C grid (R, C <= 1e7); a cell's score
//   is the sum of Manhattan distances to all tokens (<= 2e9). Ask "? p q" for a score at most K
//   times (K <= 170, 75 in the last subtask), then print "! Z", the minimum score.
// Solution: the score is a convex function of the row plus one of the column, so the best row
//   does not depend on the column: golden-section search (ternary_min) down column 1 for the
//   best row, then along that row for the best column. Probes are reused and scores cached,
//   so it asks about 2 log_phi 1e7 + 6 = 73 questions.
// original: corpus/submissions/4801866.cpp
// Bounds: rows, cols: 1..=1e7; k: 1..=170
let rows: 1..=1e7 = read()
let cols: 1..=1e7 = read()
let k: 1..=170 = read()

var seen: HashMap<(Int, Int), Int> = []
fn score(r: Int, c: Int) -> Int:
    if let known = seen.get((r, c)): return known
    let s: 0..=2e9 = ask("?", r, c)
    seen[(r, c)] = s
    s

fn cut1(l: Int, r: Int) -> Int = l + (r - l) * 38_197 / 100_000
fn cut2(l: Int, r: Int) -> Int = l + (r - l) * 61_803 / 100_000

fn ternary(n: Int, f: Int -> Int) -> Int:
    var (l, r) = (1, n)
    var (m1, m2) = (cut1(l, r), cut2(l, r))
    while m1 < m2:
        let (x1, x2) = (f(m1), f(m2))
        if x1 > x2:
            l = m1
            m1 = m2
            m2 = cut2(l, r)
        elif x1 < x2:
            r = m2
            m2 = m1
            m1 = cut1(l, r)
        else:
            (l, r) = (m1, m2)
            (m1, m2) = (cut1(l, r), cut2(l, r))
        if m1 > m2: (m1, m2) = (cut1(l, r), cut2(l, r))
    var best = l
    for i in l + 1..=r:
        if f(i) < f(best): best = i
    best

let r = ternary(rows, i => score(i, 1))
let c = ternary(cols, j => score(r, j))
print("!", score(r, c))

// Source: DMOPC 2018 Contest 1 P6 (Sorting), https://dmoj.ca/problem/dmopc18c1p6
// Problem: n <= 1000 binary numbers of exactly m <= 50 bits each, in a list. Flip the fewest
//   bits so that the list is sorted in non-decreasing order; print that count.
// Solution: sorted rows that agree before column c form a block whose bit c is 0s then 1s.
//   Going from the last column to the first, fewest[a, b] is the least flips that sort rows
//   a..b (between cuts a and b) on columns c..: choose the cut s where the block turns from 0
//   to 1, pay the wrong bits of column c on both sides (prefix counts) plus the next column's
//   fewest on a..s and s..b. The best cut is monotone in a and b (Knuth), as the accepted
//   solution uses, so each column is O(n^2): O(m n^2) overall.
// original: corpus/submissions/4155900.cpp
let n: 1..=1000 = read()
let m: 1..=50 = read()
domain Row = 0..n
domain Col = 0..m
let bits: [Row, Col]Char = read()
let column_ones = [Col] c => Prefix([Row] r => Int(bits[r, c] == '1'))
// flat, so the O(m n^2) loop reads one table
let ones_before: [Col, Row.cuts]Int = [Col, Row.cuts] (c, k) => column_ones[c].at(k)

fn flips(c: Col, a: Row.cuts, s: Row.cuts, b: Row.cuts) -> Int =
    (ones_before[c, s] - ones_before[c, a]) + (Int(b) - Int(s)) - (ones_before[c, b] - ones_before[c, s])

var fewest: [Row.cuts, Row.cuts]Int = 0
for c in Col.rev:
    var here: [Row.cuts, Row.cuts]Int = 0
    var turn: [Row.cuts, Row.cuts]Row.cuts = Row.cuts.first!
    for span in (2..=n).rev:
        for a in Row.cuts:
            if let b = Row.cuts.at(Int(a) + span):
                let lo = if let wider = a.prev: max(a, turn[wider, b]) else: a
                let hi = if let wider = b.next: min(b, turn[a, wider]) else: b
                let (cost, s) = min((flips(c, a, s, b) + fewest[a, s] + fewest[s, b], s) for s in lo..=hi)!
                here[a, b] = cost
                turn[a, b] = s
    fewest = here
print(fewest[Row.cuts.first!, Row.cuts.last!])

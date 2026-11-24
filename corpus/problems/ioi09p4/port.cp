// Source: IOI 2009 P4 (Raisins), https://dmoj.ca/problem/ioi09p4
// Problem: an n x m chocolate (n, m <= 50), 1..=1000 raisins per piece. Cut it into single pieces
//   with straight end-to-end cuts of one block at a time; each cut costs the raisins on the block
//   being cut. Print the least total cost.
// Solution: pay(r1, r2, c1, c2), memoised over all O(n^2 m^2) sub-blocks, is 0 for one piece, else
//   the block's raisins (2D prefix sums) plus the best split by a row cut or a column cut.
//   O(n^2 m^2 (n + m)).
// original: corpus/submissions/4817803.cpp
// Bounds: n, m: 1..=50; raisins: 1..=1000
let n: 1..=50 = read()
let m: 1..=50 = read()
domain Row = 1..=n
domain Col = 1..=m
let raisins: [Row, Col]1..=1000 = read()
let pre = prefix(raisins)

@memo
fn pay(r1: Row, r2: Row, c1: Col, c2: Col) -> Int:
    if r1 == r2 and c1 == c2: return 0
    let by_row = min(pay(r1, r, c1, c2) + pay(r.next!, r2, c1, c2) for r in r1..r2)
    let by_col = min(pay(r1, r2, c1, c) + pay(r1, r2, c.next!, c2) for c in c1..c2)
    min(by_row ?? by_col!, by_col ?? by_row!) + pre.fold(r1..=r2, c1..=c2)
print(pay(Row.first!, Row.last!, Col.first!, Col.last!))

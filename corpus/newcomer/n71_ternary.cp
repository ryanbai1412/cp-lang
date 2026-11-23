let n: 1..=100000 = read()
let a: [0..n]Int = read()
fn cost(x: Int) -> Int = sum(abs(v - x) for v in a)
var lo = -1000000000
var hi = 1000000000  // expect: repeated_literal
while hi - lo > 2:  // expect: missing ternary_search_by_hand
    let m1 = lo + (hi - lo) / 3
    let m2 = hi - (hi - lo) / 3
    if cost(m1) < cost(m2):
        hi = m2
    else:
        lo = m1
var best = cost(lo)
for x in lo..=hi:  // expect: loop_reduction
    best = min(best, cost(x))
print(best)

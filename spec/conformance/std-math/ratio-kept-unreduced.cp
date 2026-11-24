// expect: ok, output "-3 2 -3/2 2 1 YES YES 1/2 YES"
// A Ratio keeps the fraction it was built from; equality, hashing, `.p`, `.q` and `show`
// still read it in lowest terms, so the unreduced form never shows.
use std.math.{Ratio}
let r = Ratio(6, -4)
let keys = Set([Ratio(1, 2), Ratio(2, 4), Ratio(-3, -6), Ratio(1, 3)])
var count: Map<Ratio, Int> = Map()
for x in [Ratio(2, 6), Ratio(1, 3), Ratio(-5, -15)]: count[x] += 1
let sorted = [Ratio(4, 8), Ratio(1, 3), Ratio(2, 6), Ratio(-1, -2)].sorted()
print(
    r.p,
    r.q,
    r,
    keys.len,
    count.len,
    yes(count[Ratio(1, 3)] == 3),
    yes(sorted[0] == Ratio(1, 3) and sorted[1].p == 1 and sorted[1].q == 3),
    Ratio(3, 6) * Ratio(4, 4),
    yes(Ratio(1, 2) <= Ratio(2, 4) and not (Ratio(2, 4) < Ratio(1, 2))),
)

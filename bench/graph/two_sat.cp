// 2-SAT over n variables and m clauses `a = x or b = y`; prints how many
// variables the assignment sets, or -1 if there is none.
use std.graph.{Lit, TwoSat}
let (n, m): (1..=1e6, 0..=2e6) = read()
domain X = 0..n
var s = TwoSat<X>.new()
for _ in 0..m:
    let (a, x, b, y): (X, 0..=1, X, 0..=1) = read()
    s.add_or(Lit(a, x == 1), Lit(b, y == 1))
if let r = s.solve():
    var count = 0
    for x in X:
        if r[x]: count += 1
    print(count)
else:
    print(-1)

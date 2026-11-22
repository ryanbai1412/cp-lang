// expect: E0603 at 4
// rule: 05.7.10
let rows = List(List(1, 2), List(3))
var best = List<Int>()
for r in rows:
    if r.len > best.len: best = r      // error: r is lent for one iteration; write r.clone()

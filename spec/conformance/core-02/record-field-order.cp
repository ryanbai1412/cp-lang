// expect: E0302 at 3
// rule: 02.5.2
fn span(r: (lo: Int, hi: Int)) -> Int = r.hi - r.lo
let b = (hi: 7, lo: 2)
print(span(b))

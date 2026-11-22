// expect: E0302 at 2
// rule: 02.5.4
fn span(r: (lo: Int, hi: Int)) -> Int = r.hi - r.lo
print(span((lo: 1, hi: 4, w: 9)))

// expect: ok, output "5"
// rule: 02.5.2
fn span(r: (lo: Int, hi: Int)) -> Int = r.hi - r.lo
let a = (lo: 2, hi: 7)
print(span(a))

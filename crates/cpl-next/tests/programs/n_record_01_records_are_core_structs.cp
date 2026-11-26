// expect: ok, output "5 2\ntrue false\n9 4\n3"
// rule: R02.5.2, R02.5.5, R02.1.1
// A record is the core struct for its field names: built, read, compared, assigned and opened.
fn span(r: (lo: Int, hi: Int)) -> Int = r.hi - r.lo
let r = (lo: 2, hi: 7)
print(span(r), r.lo)
print(r == (lo: 2, hi: 7), r < (lo: 1, hi: 9))
var b = r
b.lo = 4
let (lo:, hi: top) = (lo: 9, hi: b.lo)
print(lo, top)
var t = 0
for (u:, ..) in [(u: 1, v: 5), (u: 2, v: 6)]: t += u
print(t)

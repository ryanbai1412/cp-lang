// expect: E0302
// rule: 04.4.24
// has: "`[x for x in g()]`"
fn g() -> List<Int> = [500]
var ys: List<0..100> = g() // <- here
print(ys[0])

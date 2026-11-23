// expect: ok, output "3 2"
// rule: 01.10.2
// The derivable spellings of the E0103 cases: `= expr`, `-> lends`, `enum D = A | B`.
enum Dir = N | E
domain V = 1..=4
var a: [V]Int = 1
fn tail(xs: [V]Int, l: V) -> lends [V]Int = xs[l..]
var x: Int = 0
for d in Dir: x += 1
print(tail(a, V(2)).sum(), x)

// expect: ok, output "10 4\n2"
// rule: R02.9
// input: "4\n"
// A fn and a struct may name a top-level domain, constant or read at run time.
domain P = 0..3
let n = read<1..=10>()
domain V = 1..=n
struct At:
    v: V
fn total(xs: [V]Int) -> Int:
    var s = 0
    for v in V: s += xs[v]
    s
fn last_of(a: At) -> Int = Int(a.v)
var xs: [V]Int = 0
for v in V: xs[v] = Int(v)
print(total(xs), last_of(At(V.last!)))
print(Int(P.last!))

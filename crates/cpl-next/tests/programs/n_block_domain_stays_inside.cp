// expect: ok, output "6\n2"
// rule: R02.9.2
// A domain made in a block may be used anywhere inside it; only a value of it leaving the block
// is E0313. Here each value of `V` and `W` becomes an `Int` before the block ends.
fn f(n: Int) -> Int:
    domain V = 0..n
    var s = 0
    for v in V: s += Int(v)
    s
print(f(4))
let q = { domain W = 0..3; Int(W(2)) }
print(q)

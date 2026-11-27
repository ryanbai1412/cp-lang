// expect: ok, output "5\n6\n6"
// rule: R02.5.1
// A tuple field holding a fn is called like a struct field: `t.1()` reads `_1` and calls it.
struct P(n: Int, f: () -> ())
let p = P(1, () => print(5))
p.f()
let t = (1, () => print(6))
t.1()
let ts = [t]
ts[0].1()

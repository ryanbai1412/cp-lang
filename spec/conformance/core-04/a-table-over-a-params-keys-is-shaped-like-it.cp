// expect: ok, output "6 2 4 3\n7 7 7\n6 3\n7\n10 30\n4 4 4 3"
// engines: next
// rule: 04b.4.2
// R04b.4.2: `[a.keys] k => ..` and `[a.keys].filled(x)` are tables over exactly `a`'s keys, shaped
// like `a`: a list at `D = Int`, an `[V]T` at a domain, with no count of an open `D`.
fn dbl<D: Discrete>(a: [D]Int) -> [D]Int = [a.keys] k => a[k] + a[k]
fn sevens<D: Discrete>(a: [D]Int) -> [D]Int = [a.keys].filled(7)
let xs = [3, 1, 2]
let d = dbl(xs)
print(d, d.len)
print(sevens(xs))
domain V = 0..3
let av: [V]Int = [V] v => Int(v) + 1
let dv = dbl(av)
print(dv[V(2)], dv.len)
print(sevens(av)[V(1)])
let u = [av.keys] k => av[k] * 10
print(u[V(0)], u[V(2)])
let t = [xs.keys].filled(4)
print(t, t.len)

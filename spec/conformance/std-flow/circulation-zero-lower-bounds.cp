// expect: ok, output "0 0"
domain V = 0..2
domain E = 0..2
let es: [E](u: V, v: V, lo: Int, hi: Int) = [(u: V(0), v: V(1), lo: 0, hi: 4), (u: V(1), v: V(0), lo: 0, hi: 4)]
let f = circulation(es)!
print(f[E(0)], f[E(1)])

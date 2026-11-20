type Z = Mod<7>
domain V = 0..2
let a: [V, V]Z = [V, V] (i, j) => Z(i.ord + j.ord + 1)
let c: [V, V]Z = [V, V] (i, j) => sum(a[i, m] * a[m, j] for m in V)
print(c)
var t: Z = sum(a[V(0), m] for m in V)
t += sum(a[m, m] * 2 for m in V)
print(t)

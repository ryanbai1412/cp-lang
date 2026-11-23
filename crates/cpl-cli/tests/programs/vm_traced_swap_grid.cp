// A traced run of `swap` on two cells of a 2-D packed array (`[V, V]Int`)
// stays on the VM.
let n: 1..=6 = read()
domain V = 0..n
var g: [V, V]Int = [V, V] (i, j) => Int(i) * 10 + Int(j)
for i in V:
    for j in V:
        if i < j: swap(mut g[i, j], mut g[j, i])
for i in V: print([g[i, j] for j in V])

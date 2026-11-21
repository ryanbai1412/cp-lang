use std.seq.{inverse}

@!multitest
let n: Int = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()
let b: [I]V = read()

let inv_a = a.inverse()
let a_to_b = [V] v => b[inv_a[v]]
let fixed = count(a_to_b[v] == v for v in V)
if fixed > 1 or any(a_to_b[a_to_b[v]] != v for v in V):
    print(-1)
    return

var p: [I]?Int = none
var cur = 0
for i in I if p[i] == none:
    if a[i] == b[i]:
        p[i] = n / 2
    else:
        p[i] = cur
        p[inv_a[b[i]]] = n - 1 - cur
        cur += 1

var perm = [0..n] i => p[I(i)]!
var ops: List<(Int, Int)> = []
for i in 0..n:
    while perm[i] != i:
        let j = perm[i]
        ops.push((i + 1, j + 1))
        perm.swap(i, j)

print(ops.len)
for (u, v) in ops: print(u, v)

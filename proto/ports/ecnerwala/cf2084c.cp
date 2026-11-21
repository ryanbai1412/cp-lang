@!multitest
let n: Int = read()
let a = [0..n] _ => read::<Int>() - 1
let b = [0..n] _ => read::<Int>() - 1

var a_to_b: [0..n]Int = 0
for i in 0..n: a_to_b[a[i]] = b[i]
let fixed = count(a_to_b[i] == i for i in 0..n)
if fixed > 1 or any(a_to_b[a_to_b[i]] != i for i in 0..n):
    print(-1)
    return

var inv_a: [0..n]Int = 0
for i in 0..n: inv_a[a[i]] = i
var p: [0..n]?Int = none
var cur = 0
for i in 0..n if p[i] == none:
    if a[i] == b[i]:
        p[i] = n / 2
    else:
        p[i] = cur
        p[inv_a[b[i]]] = n - 1 - cur
        cur += 1

var perm = [0..n] i => p[i]!
var ops: List<(Int, Int)> = []
for i in 0..n:
    while perm[i] != i:
        let j = perm[i]
        ops.push((i + 1, j + 1))
        perm.swap(i, j)

print(ops.len)
for (u, v) in ops: print(u, v)

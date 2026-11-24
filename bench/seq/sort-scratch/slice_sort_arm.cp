let n = 200_000
var xs: List<Int> = [0 for _ in 0..n]
var seed = 123456789
for i in 0..n:
    seed = (seed * 1103515245 + 12345) % 2147483647
    xs[i] = seed

var acc = 0
for r in 0..40:
    let base = (r * 3713) % (n / 2)
    let len = n / 2
    let l = base
    let hi = base + len
    xs[l..hi].sort()
    acc += xs[l] + xs[l + len / 2] + xs[hi - 1]
    // Scramble the middle so the next round sorts non-trivial data.
    for i in l..hi:
        seed = (seed * 1103515245 + 12345) % 2147483647
        xs[i] = xs[i] ^ seed

print(acc)

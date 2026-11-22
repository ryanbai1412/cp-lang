// Codeforces 2101 B — Quartet Swapping
//
// Each operation swaps (a[i], a[i + 1]) with (a[i + 2], a[i + 3]), which
// performs one transposition on even indices and one on odd indices at once.
// Hence even and odd positions stay partitioned, and the XOR of their
// inversion parities is invariant (and every permutation matching both
// invariants is reachable).
//
// The lexicographically smallest reachable permutation therefore sorts even
// positions `x` and odd positions `y` independently, and if their inversion
// parities differ, swaps the last two same-parity elements (n - 3, n - 1).

@!multitest
let n: 4..=200_000 = read()
domain V = 1..=n
var a: [0..n]V = read()

fn parity(vals: List<V>) -> Int:
    var bit = Fenwick<V, Int>()
    var ans = 0
    for (seen, v) in vals.items:
        ans ^= (seen - bit.prefix(v.r)) & 1
        bit.add(v, 1)
    ans

var x = [a[i] for i in 0..n by 2]
var y = [a[i] for i in 1..n by 2]
let mismatch = (parity(x) ^ parity(y)) != 0
x.sort()
y.sort()
for (i, v) in x.items: a[2 * i] = v
for (i, v) in y.items: a[2 * i + 1] = v
if mismatch: a.swap(n - 3, n - 1)
print(a)

// CF 2050F Maximum modulo equality. a[l..=r] agree mod m exactly when m divides every
// adjacent difference there, so the answer is their gcd (0 for a single element).
@!multitest
let n, q: 1..=2e5 = read()
let a = read<[n]1..=1e9>()
let diff = SparseTable(gcd, [0..n - 1] i => abs(a[i + 1] - a[i]))
var out: List<Int> = []
repeat q:
    let l: 1..=n = read()
    let r: l..=n = read()
    out.push(diff.try_fold(l - 1..r - 1) ?? 0)
print(out)

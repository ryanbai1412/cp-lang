// Codeforces 2268 B — Even Popcount Point Updates
//
// Each non-negative value `x in 0..2^30` is good iff `popcount(x)` is even
// (the binary-parity invariant preserved by even-weight operations). Maintain
// the count `good` in O(1) per point assignment `a[p] = x` via
// `good += even(x) - even(a[p])`.

const MAX_V = (1 << 30) - 1
fn even(x: 0..=MAX_V) -> Int: Int(popcount(x) % 2 == 0)

@!multitest
let (n, q): (2..=200_000, 0..=200_000) = read()
var a: [1..=n]0..=MAX_V = read()

var good = sum(even(x) for x in a)
var out = [good]
for _ in 0..q:
    let (p, x): (1..=n, 0..=MAX_V) = read()
    good += even(x) - even(a[p])
    a[p] = x
    out.push(good)
print(out)

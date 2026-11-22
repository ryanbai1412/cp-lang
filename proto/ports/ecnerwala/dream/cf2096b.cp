// Codeforces 2096 B — Wonderful Gloves
//
// Each colour `i` has `l[i]` left and `r[i]` right gloves. To avoid forming `k`
// matched pairs in the worst case, an adversary first gives us all
// `max(l[i], r[i])` gloves from the larger side of every colour (0 pairs), then
// all `min(l[i], r[i])` gloves from the `k - 1` colours with the largest `min`
// (`k - 1` pairs). Drawing `+ 1` additional glove is guaranteed to complete a
// `k`-th matched pair.

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let l: [0..n]0..=10 ** 9 = read()
let r: [0..n]0..=10 ** 9 = read()
let mins = sorted([min(l[i], r[i]) for i in 0..n], desc: true)
print(sum(max(l[i], r[i]) for i in 0..n) + mins[0..k - 1].sum() + 1)

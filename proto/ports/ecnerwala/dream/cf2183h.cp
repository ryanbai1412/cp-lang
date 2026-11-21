// Codeforces 2183 H: Minimum Total Cost of k Subsequences.
// After sorting `a`, optimal groups are contiguous. The first group must cover
// the prefix `0..st` where `pref[c] + a[c] * c <= 0`; beyond `st`, segment cost
// `cost(j, i) = (pref[i] - pref[j]) * (i - j)` is Monge and the optimal k-group
// cost `f(k)` is convex, so Aliens binary search over a half-integer slope plus
// O(n) LARSCH (`monge_path`) finds the exact minimum.

@!multitest
let (n, k): (1..=200_000, 1..=n) = read()
let a: [0..n]-1_000_000_000..=1_000_000_000 = read().sorted()
let pref = a.prefix_sums()
let st = a.cuts.find(c => pref[c] + a[c] * c.ord > 0) ?? a.Cut.last
let most = n - st.ord + 1

fn cost(j: a.Cut, i: a.Cut) -> I128 = (pref[i] - pref[j]) * (i.ord - j.ord)

let ans =
    if k >= most:
        let rest = a.Cut(n - k + 1)
        cost(a.Cut.first, rest) + (pref[a.Cut.last] - pref[rest])
    else:
        aliens(
            groups: k,
            slope: I128,
            pen => monge_path(
                through: [a.Cut.first] + [c for c in st..=a.Cut.last],
                cost: (j, i) => cost(j, i) + pen,
            ),
        )
print(ans)

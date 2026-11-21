// Codeforces 2183 H (dream). Sorted, groups are contiguous; the first group
// takes the prefix 0..st, past it the cost is Monge and f(k) is convex.
@!multitest
let (n, k): (1..=2e5, 1..=n) = read()
let a: [n]-1e9..=1e9 = read().sorted()
let pref = a.prefix_sums()                                   // [a.cuts]Int
let st = a.cuts.first(c => pref[c] + a[c] * c > 0) ?? n
let most = n - st + 1
fn cost(j: a.Cut, i: a.Cut) -> I128 = (pref[i] - pref[j]) * (i - j)   // widens, no I128(...)
if k >= most:
    let rest = n - k + 1
    return print(cost(0, rest) + (pref[n] - pref[rest]))
print(aliens(groups: k, slope: I128) { pen =>                 // exact, unique-count ties
    monge_path(through: [0] ++ (st..=n), cost: (j, i) => cost(j, i) + pen)   // LARSCH, O(n)
})

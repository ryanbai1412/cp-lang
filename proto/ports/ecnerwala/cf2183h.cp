// Codeforces 2183 H.
// Sorted, an optimal partition is contiguous. The first group always takes
// the prefix `0..st` (each of those elements lowers its cost), and past it the
// segment cost (P_i - P_j)(i - j) is Monge, so f(k) is convex: Aliens trick.
// A group costs an extra 1/2 - lam (doubled to stay integral), which makes the
// optimal group count unique; binary search the least lam reaching k groups.
// Each penalized DP is a Monge 1D/1D DP: a deque of candidates, each owning
// the rows from where it starts to win, found by binary search.
type Best = (cost: I128, groups: Int)
@!multitest
let n: 1..=200_000 = read()
let k: 1..=n = read()
var a: [0..n]-1_000_000_000..=1_000_000_000 = read()
sort(mut a)
var pref: [0..=n]Int = 0
for i in 0..n: pref[i + 1] = pref[i] + a[i]
var st = 0
while st < n and pref[st] + a[st] * st <= 0: st += 1

fn solve(lam: I128) -> Best:
    var dp: [0..=n]Best = (cost: I128(0), groups: 0)
    fn via(
        j: Int,
        i: Int,
    ) -> I128 = dp[j].cost + 2 * I128(pref[i] - pref[j]) * (i - j) - (2 * lam - 1)
    var cand = Deque<(j: Int, from: Int)>()
    cand.push((j: 0, from: st))
    for i in st..=n:
        while cand.len >= 2 and cand[1].from <= i: cand.pop_front()
        let j = cand[0].j
        dp[i] = (cost: via(j, i), groups: dp[j].groups + 1)
        if i == n: break
        while let last = cand.last, via(i, max(last.from, i + 1)) <= via(
            last.j,
            max(last.from, i + 1),
        ):
            cand.pop()
        if let last = cand.last:
            if let r = bsearch_first(max(last.from, i + 1)..=n, r => via(i, r) <= via(last.j, r)):
                cand.push((j: i, from: r))
        else:
            cand.push((j: i, from: i + 1))
    dp[n]

let one_group = I128(pref[n]) * n
let most_groups = n - st + 1
if k == 1:
    print(one_group)
elif k >= most_groups:
    let rest = n - k + 1
    print(I128(pref[rest]) * rest + (pref[n] - pref[rest]))
else:
    let fewest = I128(pref[st]) * st + (pref[n] - pref[st]) - one_group
    var (lo, lam) = (fewest, I128(0))
    while lo < lam:
        let mid = lo + (lam - lo) / 2
        if solve(mid).groups >= k: lam = mid else: lo = mid + 1
    let best = solve(lam)
    let f = (best.cost + (2 * lam - 1) * best.groups) / 2
    print(f - (lam - 1) * (best.groups - k))

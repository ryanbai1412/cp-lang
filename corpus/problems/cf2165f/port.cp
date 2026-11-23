// Source: Codeforces 2165F (Arctic Acquisition), https://codeforces.com/contest/2165/problem/F
// Problem: a permutation a of n <= 1e6 (sum of n <= 1e6, multitest). Count the intervals [l, r]
//   that contain a 21435 subsequence: i1 < ... < i5 with a[i2] < a[i1] < a[i4] < a[i3] < a[i5].
// Solution: for each right end r let best[r] be the latest l starting a pattern that ends by r;
//   the answer sums prefix-max(best) + 1. Exchange arguments fix three picks: "5" is next_greater
//   of "3", "4" is the latest earlier position valued between "3" and "5" (SegTree of positions
//   by value), and "1" is next_less of "2". Sweeping the "4" positions, the "2"s whose "1" is
//   already passed sit in a max SegTree by value; the best "2" below "3" updates best at "5".
//   O(n log n).

@!multitest
let n: 1..=1_000_000 = read()
domain I = 0..n
domain V = 1..=n
let a: [I]V = read()

// shapes[k]: (value of "3", position of "5") for each "435" whose "4" is at k.
var shapes: [I]List<(V, I)> = []
var latest = SegTree<V, ?I>(max)
let next_g = a.next_smaller(by: x => desc(x))
for j in I:
    if let i = next_g[j], let k = latest.fold(a[j].next!..a[i]): shapes[k].push((a[j], i))
    latest.set(a[j], j)

// A "2" at j becomes active once its "1" (next_less[j]) has been passed.
var open_at: [I]List<I> = []
let next_l = a.next_smaller()
for j in I:
    if let i = next_l[j]: open_at[i].push(j)

var two = SegTree<V, ?I>(max)
var best: [I]?I = None
for i in I:
    for (v, r) in shapes[i]: best[r] max= two.fold(..v)
    for j in open_at[i]: two.set(a[j], j)

let best_so_far = best.scan(max)
print([l.ord + 1 for k in I if let l = best_so_far.prefix(k.ord + 1)].sum())

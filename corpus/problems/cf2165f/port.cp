// Codeforces 2165 F: 21435.
// Count subarrays containing a 21435 subsequence. For each right end `r`,
// `best[r]` is the latest left end of a 21435 ending by `r`; the answer is the
// sum of `best + 1` after a prefix max. Exchange arguments let "5" be
// `next_greater` of "3", "4" be the latest earlier element valued between "3"
// and "5", and "1" be `next_less` of "2".

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

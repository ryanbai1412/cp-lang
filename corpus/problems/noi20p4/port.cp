// Source: NOI 2020 P4 (Dish), https://dmoj.ca/problem/noi20p4
// limits: 2000 ms, 512 MB
// Problem: n ingredients of d_i grams (sum = m * k); make m dishes of exactly k grams, each from
//   at most 2 ingredients in positive whole grams, using everything up; print the dishes, or -1.
//   Multitest (T <= 10), n <= 500, n - 2 <= m <= 5000, k <= 5000; any valid plan.
// Solution: with m >= n - 1, greedy over a `SortedBag` of (grams left, ingredient): while
//   there are more dishes than ingredients - 1, the largest (>= k) makes a dish alone; otherwise
//   the smallest (< k) is finished off with the largest. With m = n - 2, split the ingredients into two groups of s and n - s that
//   each hold (size - 1) * k grams: a subset of offsets d_i - k summing to -k, found by a
//   subset-sum bitset per prefix, O(n * n * k / 64), traced back; then greedy on each.
// original: corpus/submissions/3890337.cpp
// Bounds: n: 1..=500; m, k: 1..=5000; d: 1..=m * k
@multitest
let (n, m, k): (1..=500, 1..=5000, 1..=5000) = read()
domain Ing = 0..n
let d: [Ing]1..=m * k = read()

// Uses up the ingredients in `group` with exactly `dishes` dishes
// (dishes >= group.len - 1 and their total is dishes * k).
fn greedy(group: List<Ing>, dishes: Int):
    var live = SortedBag.of([(d[i], i) for i in group])
    for made in 0..dishes:
        let (big, j) = live.last!
        live.remove((big, j))
        if dishes - made > live.len:
            print(j.ord + 1, k)
            if big > k: live.add((big - k, j))
        else:
            let (small, i) = live.first!
            live.remove((small, i))
            print(i.ord + 1, small, j.ord + 1, k - small)
            if big + small > k: live.add((big + small - k, j))

// Two groups, each holding (size - 1) * k grams: a subset of the offsets d - k summing to -k.
// A running sum above -k - mn (mn: all negative offsets) never comes back down, so sums live in
// mn..=-k - mn.
fn halves() -> ?(List<Ing>, List<Ing>):
    let mn = sum(min(d[i] - k, 0) for i in Ing)
    var off = -k - mn
    if off < 0: return None
    domain Off = 0..=off - mn
    var reach = [Bitset<Off>.single(Off(-mn))]  // reach[i]: subset sums of the first i, shifted by -mn
    for i in Ing: reach.push(reach.last!.bitor(reach.last!.shifted(d[i] - k)))
    if not reach[n].contains(Off(off)): return None
    var (s, rest) = (List<Ing>(), List<Ing>())
    for i in Ing.rev:
        if reach[i.ord].contains(Off(off)):
            rest.push(i)
        else:
            s.push(i)
            off -= d[i] - k
    (s, rest)

if m >= n - 1:
    greedy(Ing.collect(), m)
elif let (s, rest) = halves():
    greedy(s, s.len - 1)
    greedy(rest, rest.len - 1)
else:
    print(-1)

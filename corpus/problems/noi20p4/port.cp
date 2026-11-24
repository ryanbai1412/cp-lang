// Source: NOI 2020 P4 (Dish), https://dmoj.ca/problem/noi20p4
// limits: 2000 ms, 512 MB
// Problem: n ingredients of d_i grams (sum = m * k); make m dishes of exactly k grams, each from
//   at most 2 ingredients in positive whole grams, using everything up; print the dishes, or -1.
//   Multitest (T <= 10), n <= 500, n - 2 <= m <= 5000, k <= 5000; any valid plan.
// Solution: with m >= n - 1, greedy: while there are more dishes than ingredients - 1, the
//   largest ingredient (>= k) makes a dish alone; otherwise the smallest (< k) is finished off
//   with the largest. With m = n - 2, split the ingredients into two groups of s and n - s that
//   each hold (size - 1) * k grams: a subset of offsets d_i - k summing to -k, found by a
//   subset-sum bitset per prefix over sums in -n k..=n k, O(n * n * k / 64), traced back; then greedy on each.
// original: corpus/submissions/3890337.cpp
let tests: 1..=10 = read()

var rem: List<Int> = []  // grams left of each ingredient of this case
var k = 0

// Uses up the ingredients in `group` with exactly `dishes` dishes
// (dishes >= group.len - 1 and their total is dishes * k).
fn greedy(group: List<Int>, dishes: Int):
    var live = [i for i in group if rem[i] > 0]
    for made in 0..dishes:
        var a = 0
        var b = 0
        for p in 0..live.len:
            if rem[live[p]] < rem[live[a]]:
                a = p
            if rem[live[p]] >= rem[live[b]]:
                b = p
        let i = live[a]
        let j = live[b]
        let small = rem[i]
        if dishes - made > live.len - 1:
            print(j + 1, k)
            rem[j] -= k
        else:
            print(i + 1, small, j + 1, k - small)
            rem[i] = 0
            rem[j] -= k - small
        live = [q for q in live if rem[q] > 0]

for t in 0..tests:
    let n: 1..=500 = read()
    let m: 1..=5000 = read()
    let kk: 1..=5000 = read()
    k = kk
    rem = []
    for i in 0..n:
        let d: Int = read()
        rem.push(d)
    if m >= n - 1:
        greedy((0..n).collect(), m)
    else:
        // offsets d - k summed over a subset; find a subset summing to -k. A running sum above
        // -k - mn (mn: all negative offsets) can never come back down, so sums live in mn..=-k - mn.
        let mn = sum(min(d - k, 0) for d in rem)
        var off = -k - mn
        if off < 0:
            print(-1)
            continue
        domain Off = 0..=off - mn
        // reach[i]: the sums (shifted by -mn) of subsets of the first i offsets
        var reach = [Bitset<Off>.single(Off(-mn))]
        for i in 0..n: reach.push(reach[i].bitor(reach[i].shifted(rem[i] - k)))
        if not reach[n].contains(Off(off)):
            print(-1)
        else:
            var s: List<Int> = []
            var rest: List<Int> = []
            for back in 0..n:
                let i = n - 1 - back
                if reach[i].contains(Off(off)):
                    rest.push(i)
                else:
                    s.push(i)
                    off -= rem[i] - k
            greedy(s, s.len - 1)
            greedy(rest, rest.len - 1)

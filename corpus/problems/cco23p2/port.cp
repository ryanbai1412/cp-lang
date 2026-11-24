// Source: CCO 2023 Day 1 P2, Real Mountains, https://dmoj.ca/problem/cco23p2
// limits: 5000 ms, 1024 MB
// Problem: heights h_1..h_N (N <= 1e6, h_i <= 1e9). An edit picks i < j < k with
//   h_i > h_j < h_k, costs h_i + h_j + h_k and increments h_j. Print the minimum total cost
//   to make h unimodal (nondecreasing, then nonincreasing), mod 1e6 + 3.
// Solution: column i must end at cap_i = min(prefix max, suffix max). Sweep the distinct
//   heights upwards, keeping the columns still below their cap in a SortedSet. Lifting them
//   from level x costs x per column plus the least heights above x left of the first and right
//   of the last (one column), or (2sz - 3)(x + 1) plus those and the least above x overall
//   (sz >= 2). Level sums use x(x+1)/2; the least-above-x terms are deferred to a prefix and a
//   suffix sweep of successor queries on a SortedSet. O(N log N).
// original: corpus/submissions/5579928.cpp

type Z = Mod<1_000_003>
let n: 1..=1e6 = read()
domain Col = 0..n
let h: [Col]1..=1e9 = read()
let (left, right) = (h.scan(max), h.rev.scan(max))
let cap = [Col] c => min(left[c], right[c])
let (domain Level, height) = compress(h)
let at = ([Col] c => height.index_of(h[c])).buckets()

fn tri(x: Int) -> Z = Z(x) * Z(x + 1) / Z(2)
fn heights(lo: Int, hi: Int) -> Z = tri(hi) - tri(lo - 1)

var from_left: [Col]List<(x: Int, times: Int)> = []
var from_right: [Col]List<(x: Int, times: Int)> = []
var low = SortedSet<Col>()
var cost = Z(0)
for l in Level:
    let Some(u) = l.next else: break
    let (x, y) = (height[l], height[u])
    for c in at[l]: low.add(c)
    while low.first.is_some_and(c => cap[c] <= x): low.pop_first()
    while low.last.is_some_and(c => cap[c] <= x): low.pop_last()
    let Some(a) = low.first else: continue
    let b = low.last!
    let ask = (x:, times: y - x)
    from_left[a].push(ask)
    if a == b:
        cost += heights(x, y - 1)
        from_right[a].push(ask)
    else:
        cost += Z(low.len) * heights(x, y - 1) + Z(2 * low.len - 3) * heights(x + 1, y)
        from_right[b].push(ask)
        from_left[Col.last!].push(ask)

fn least_above(order: List<Col>, asks: [Col]List<(x: Int, times: Int)>) -> Z:
    var seen = SortedSet<Int>()
    var sum = Z(0)
    for c in order:
        seen.add(h[c])
        for q in asks[c]: sum += Z(q.times) * Z(seen.range(q.x + 1..).first!)
    sum

print(cost + least_above(Col.collect(), from_left) + least_above(Col.rev.collect(), from_right))

// Tuned: port.cp with the tops as a flat max tree over the skills present (a top is the first
// present sorcerer above every skill before it), searched by an iterative descent, as orig.cpp's
// in-order segment tree, with a Fenwick count of tops, instead of a SortedSet of tops.
// Source: Codeforces 2262B (Culling Game), https://codeforces.com/contest/2262/problem/B
// limits: 2000 ms, 256 MB
// Problem: n <= 2e5 sorcerers (multitest, sum n <= 2e5), skills a_i <= 1e9. The leftmost is
//   champion and meets the rest left to right: if his skill s < x he forfeits and x takes
//   over, else he wins and s += x. For each i = 0..n-1, after removing sorcerers p_1..p_i
//   (p a permutation), print the number of forfeits.
// Solution: only a prefix maximum (a top) can become champion, and the answer is (#tops - 1)
//   minus the tops some champion beats. An eater c beats the next top; it absorbs tops until one
//   exceeds its running sum, which at least doubles per top, so its walk takes O(log max a)
//   searches. Answer removals backwards as insertions, with a Fenwick tree of skills and sorted
//   sets of the tops and the eaters.
// Bounds: n: 1..=200_000; a: 1..=1_000_000_000; p: 1..=n
@!multitest
let n: 1..=200_000 = read()
domain I = 1..=n
let a: [I]1..=1_000_000_000 = read()
let p: [0..n]I = read()

var total = Fenwick<I, Int>()
let size = bit_ceil(n)
var mx: List<Int> = List.filled(2 * size, 0)  // max skill present below each node, 0 for none

fn put(i: I, v: Int):
    var x = i.ord + size
    while x >= 1 and mx[x] < v:
        mx[x] = v
        x >>= 1

fn max_below(i: I) -> Int:
    var (l, r, best) = (size, i.ord + size, 0)
    while l < r:
        if l % 2 == 1:
            best max= mx[l]
            l += 1
        if r % 2 == 1:
            r -= 1
            best max= mx[r]
        (l, r) = (l / 2, r / 2)
    best

var top = Fenwick<I, Int>()
var tops = 0
var eaters = SortedSet<I>()

// The first top right of `from` whose skill exceeds `x`, given `x` is at least every skill left of `from`.
fn next_above(from: I.bounds, x: Int) -> ?I:
    if from == I.end: return None
    var v = from.ord + size
    while true:
        while v % 2 == 0: v /= 2
        if mx[v] > x:
            while v < size:
                v *= 2
                if mx[v] <= x: v += 1
            return Some(I.nth(v - size))
        v += 1
        if v & -v == v: return None

fn recheck(c: I):
    eaters.set(c, on: next_above(c.r, a[c]).map(q => total.fold(c..q) >= a[q]) ?? false)

fn insert(i: I):
    total.add(i, a[i])
    let m = max_below(i)
    let before = if m == 0: None else: next_above(I.start, m - 1)
    put(i, a[i])
    if let c = before and a[c] >= a[i]:
        recheck(c)
        return
    var q = next_above(i.r, m)
    while let t = q and a[t] <= a[i]:
        top.add(t, -1)
        tops -= 1
        eaters.remove(t)
        q = next_above(t.r, a[t])
    top.add(i, 1)
    tops += 1
    if let c = before: recheck(c)
    recheck(i)

// Where eater c's run ends: grow past every top its sum through `stop` beats.
fn reach(c: I, stop: I.bounds) -> I.bounds:
    let nxt = next_above(c.r, total.fold(c.l..stop)).map(q => q.l) ?? I.end
    if nxt == stop: stop else: reach(c, nxt)

fn forfeits() -> Int:
    var lost = tops - 1
    var from = I.first
    while let c = eaters.range(from!..).first:
        let stop = reach(c, next_above(c.r, a[c]).map(q => q.l) ?? I.end)
        lost -= top.prefix(stop) - top.prefix(c.l) - 1
        if let s = stop.right: from = s else: break
    lost

var ans: [0..n]Int = 0
for k in (0..n).rev:
    insert(p[k])
    ans[k] = forfeits()
print(ans)

// A multiset of barrels can merge to one iff no threshold k >= 1 traps barrels >= k:
// specifically, no k with >= 2 barrels >= k, sum of barrels < k strictly < k, and
// no adjacent gap < k occurring at or above k. Split at B = 1000 = sqrt(max a):
// k > B walks down from max(cur) in O(sqrt(M)) steps; k <= B sweeps tracked small gaps.
let (n, q): (1..=200_000, 1..=200_000) = read()
let B = 1000
domain Small = 0..=B

var bag = SortedBag<Int>()
var gaps: [Small]SortedBag<Int> = [Small] _ => SortedBag<Int>()
var last_gap_at: [Small]Int = 0
var small_cnt: [Small]Int = 0
var total = 0

fn update_gap(lo: Int, hi: Int, add: Bool):
    let l = hi - lo
    if l > B: return
    let d = Small(l)
    if add: gaps[d].add(lo) else: gaps[d].remove(lo)
    last_gap_at[d] = gaps[d].last ?? 0

fn change(v: Int, add: Bool):
    let d = if add: 1 else: -1
    let had = bag.count(v)
    if add: bag.add(v) else: bag.remove(v)
    if had >= (if add: 1 else: 2):
        update_gap(v, v, add)
    else:
        let prv = bag.range(..v).last
        let nxt = bag.range(v + 1..).first
        if let p = prv: update_gap(p, v, add)
        if let x = nxt: update_gap(v, x, add)
        if let p = prv:
            if let x = nxt: update_gap(p, x, not add)
    if v <= B: small_cnt[Small(v)] += d
    total += d * v

fn can_merge() -> Bool:
    var cur = bag.last!
    if bag.count(cur) == 1:
        var rem = total - cur
        var min_gap = cur
        while let nxt = bag.range(..cur).last:
            rem -= nxt
            min_gap = min(min_gap, min(cur - nxt, nxt))
            if rem < min_gap: return false
            if min_gap <= B or bag.count(nxt) >= 2: break
            cur = nxt

    var max_gap_pos = 0
    var big_cnt = bag.len
    var small_sum = 0
    for k in 1..=B:
        let d = Small(k - 1)
        max_gap_pos max= last_gap_at[d]
        big_cnt -= small_cnt[d]
        small_sum += (k - 1) * small_cnt[d]
        if big_cnt < 2: break
        if max_gap_pos < k and small_sum < k: return false
    true

type Vol = 1..=1_000_000

repeat n:
    let a: Vol = read()
    change(a, true)
print(if can_merge(): "Yes" else: "No")

repeat q:
    let op: Char = read()
    let a: Vol = read()
    change(a, op == '+')
    print(if can_merge(): "Yes" else: "No")

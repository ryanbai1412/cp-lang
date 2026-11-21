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

fn add_barrel(v: Int):
    total += v
    if v <= B: small_cnt[Small(v)] += 1
    let had = bag.count(v)
    bag.add(v)
    if had >= 1:
        update_gap(v, v, true)
    else:
        let prv = bag.last_lt(v)
        let nxt = bag.first_gt(v)
        if let p = prv:
            if let x = nxt: update_gap(p, x, false)
            update_gap(p, v, true)
        if let x = nxt: update_gap(v, x, true)

fn remove_barrel(v: Int):
    let had = bag.count(v)
    bag.remove(v)
    if had >= 2:
        update_gap(v, v, false)
    else:
        let prv = bag.last_lt(v)
        let nxt = bag.first_gt(v)
        if let p = prv: update_gap(p, v, false)
        if let x = nxt: update_gap(v, x, false)
        if let p = prv:
            if let x = nxt: update_gap(p, x, true)
    if v <= B: small_cnt[Small(v)] -= 1
    total -= v

fn can_merge() -> Bool:
    var cur = bag.last!
    if bag.count(cur) == 1:
        var rem = total - cur
        var min_gap = cur
        while let nxt = bag.last_lt(cur):
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
    add_barrel(a)
print(if can_merge(): "Yes" else: "No")

repeat q:
    let op: Char = read()
    let a: Vol = read()
    if op == '+': add_barrel(a) else: remove_barrel(a)
    print(if can_merge(): "Yes" else: "No")

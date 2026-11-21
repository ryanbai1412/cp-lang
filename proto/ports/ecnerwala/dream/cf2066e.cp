// A multiset of barrels can merge to one iff no threshold k >= 1 traps barrels >= k:
// specifically, no k with >= 2 barrels >= k, sum of barrels < k strictly < k, and
// no adjacent gap < k occurring at or above k. Split at B = 1000 = sqrt(max a):
// k > B walks down from max(cur) in O(sqrt(M)) steps; k <= B sweeps tracked small gaps.
let (n, q): (1..=2e5, 1..=2e5) = read()
const B = 1000
domain Small = 0..=B

struct Barrels:
    bag = SortedBag<Int>()
    gaps: [Small]SortedBag<Int> = [Small] _ => SortedBag<Int>()
    small_cnt: [Small]Int = 0
    total = 0

    fn update_gap(mut self, lo: Int, hi: Int, add: Bool):
        if hi - lo > B: return
        let d = Small(hi - lo)
        if add: gaps[d].add(lo) else: gaps[d].remove(lo)

    fn add(mut self, v: Int):
        total += v
        if v <= B: small_cnt[Small(v)] += 1
        let had = bag.count(v)
        bag.add(v)
        if had >= 1:
            update_gap(v, v, true)
        else:
            let (prv, nxt) = (bag.last_lt(v), bag.first_gt(v))
            if let (p, x) = (prv, nxt): update_gap(p, x, false)
            if let p = prv: update_gap(p, v, true)
            if let x = nxt: update_gap(v, x, true)

    fn remove(mut self, v: Int):
        let had = bag.count(v)
        bag.remove(v)
        if had >= 2:
            update_gap(v, v, false)
        else:
            let (prv, nxt) = (bag.last_lt(v), bag.first_gt(v))
            if let p = prv: update_gap(p, v, false)
            if let x = nxt: update_gap(v, x, false)
            if let (p, x) = (prv, nxt): update_gap(p, x, true)
        if v <= B: small_cnt[Small(v)] -= 1
        total -= v

    fn can_merge(self) -> Bool:
        // Large k > B: walk down from the largest barrel using O(1) cursor steps.
        var cur = bag.last!
        if bag.count(cur) == 1:
            var rem = total - cur
            var min_gap = cur
            for nxt in bag.cursor(cur).prevs():
                rem -= nxt
                min_gap = min(min_gap, cur - nxt, nxt)
                if rem < min_gap: return false
                if min_gap <= B or bag.count(nxt) >= 2: break
                cur = nxt
        // Small k <= B: sweep gap lengths 0..B.
        var max_gap_pos = 0
        var big_cnt = bag.len
        var small_sum = 0
        for k in 1..=B:
            let d = Small(k - 1)
            max_gap_pos max= gaps[d].last ?? 0
            big_cnt -= small_cnt[d]
            small_sum += (k - 1) * small_cnt[d]
            if big_cnt < 2: break
            if max_gap_pos < k and small_sum < k: return false
        true

var barrels = Barrels()
for v in read<[n]1..=1e6>(): barrels.add(v)
print(if barrels.can_merge(): "Yes" else: "No")

repeat q:
    let (op, v): (Char, 1..=1e6) = read()
    if op == '+': barrels.add(v) else: barrels.remove(v)
    print(if barrels.can_merge(): "Yes" else: "No")

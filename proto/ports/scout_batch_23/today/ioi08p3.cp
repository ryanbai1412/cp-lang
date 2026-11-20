// IOI '08 P3 - Fish (runnable on proto/cp.py lib-run today)

let f_cnt: Int = read()
let k: Int = read()
let M: Int = read()
type Z = Mod<M>

var raw: List<(Int, Int)> = []
for _ in 0..f_cnt:
    let l: Int = read()
    let g: Int = read()
    raw.push((l, g))

var max_len = List.filled(k + 1, 0)
for idx in 0..f_cnt:
    let (l, g) = raw[idx]
    if l > max_len[g]:
        max_len[g] = l

var gem_pairs: List<(Int, Int)> = []
for g in 1..=k:
    gem_pairs.push((max_len[g], g))
gem_pairs.sort()

var rank_of = List.filled(k + 1, 0)
var max_by_rank = List.filled(k, 0)
for r in 0..k:
    let (ml, g) = gem_pairs[r]
    rank_of[g] = r
    max_by_rank[r] = ml

var fish: List<(Int, Int)> = []
for idx in 0..f_cnt:
    let (l, g) = raw[idx]
    fish.push((l, rank_of[g]))
fish.sort()

domain Gem = 0..k
var cnt = List.filled(k, 0)
var seg = SegTree<Gem, Z>.of(*, _ => Z(1))

var p = 0
var ans: Z = 0
for r in 0..k:
    let lim = max_by_rank[r] / 2
    while p < f_cnt and fish[p].0 <= lim:
        let g = fish[p].1
        cnt[g] += 1
        seg.set(Gem(g), Z(cnt[g] + 1))
        p += 1

    let ways_small = if r > 0: seg.fold(Gem(0)..=Gem(r - 1)) else: Z(1)

    // Find first rank >= r whose max length >= 2 * lim_next? No: find last rank q >= r
    // whose fish with max_by_rank[q] < 2 * fish_len? Wait: exact condition for gem r to be
    // the largest gem in a bag: we eat a largest fish of gem r (len = max_by_rank[r]), so
    // any other fish of gem g < r has len <= max_by_rank[r]/2 (counted in seg[0..r-1]).
    // For gems q > r, we CANNOT pick a fish of gem q that has len > max_by_rank[r] (else q
    // would be larger!), AND we can pick at most cnt[q] fishes of gem q with len <= max_by_rank[r]/2;
    // wait — if we pick a fish of gem r of length L_r (smaller than max_by_rank[r]), we could
    // avoid blocking gem q? NO: if we pick a fish of gem q with len > max_by_rank[r]/2, then
    // since max_by_rank[q] >= max_by_rank[r], gem q's max fish is ALSO >= max_by_rank[r], so
    // wait: every gem q > r has max_by_rank[q] >= max_by_rank[r], so any subset with max gem r
    // only picks fishes from gems <= r, PLUS fishes of gem q > r that are <= max_by_rank[r]/2
    // PROVIDED we pick a fish of gem r that is >= 2 * (any fish of gem q) AND no fish of gem q
    // can exceed max_by_rank[r] ... Wait: IOI 08 Fish counts DISTINCT sets of gem counts!
    // Let's check exact condition!
    // For gem r, let t = smallest index with max_by_rank[t] >= 2 * fish_of_r_above_lim?
    // Let's keep our clean two-case split:
    // Case A: pick ALL cnt[r] small fish of gem r plus the max fish of gem r -> (cnt[r] + 1)
    //   wait: we can pick 1..=cnt[r] small fish of gem r (that gives ways_small * cnt[r] when
    //   we don't exceed any gem q > r), OR pick the big fish of gem r!
    //   Let's check exact upper bound for Case B (max fish of gem r is eaten):
    //   Among gems q > r, a gem q is ALREADY counted if we only choose from small fish of q <= lim
    //   EXCEPT if gem q has a fish of length in (lim, max_by_rank[r]] — wait: if we choose NO
    //   fish from gem q of length > lim, then max_by_rank[q] >= max_by_rank[r] means we COULD
    //   have replaced our choice of small fishes of gem q with gem q's max fish... UNLESS
    //   gem q has NO fish with length >= 2 * (some fish of gem r we must eat) i.e. unless
    //   max_by_rank[q] < 2 * next_fish_of_r!
    // Let's find `next_r` = smallest length of a fish of gem r that is > lim:
    var next_r = max_by_rank[r]
    // Find via scan over fish[p..f_cnt] with gem == r (or precomputed)
    var q_hi = r
    var lo = r + 1
    var hi = k
    // Find first fish of gem r at index >= p
    var found = false
    var idx = p
    while idx < f_cnt:
        if fish[idx].1 == r:
            next_r = fish[idx].0
            found = true
            break
        idx += 1
    while lo < hi:
        let mid = (lo + hi) / 2
        if max_by_rank[mid] < 2 * next_r:
            lo = mid + 1
        else:
            hi = mid
    q_hi = lo - 1

    // Case 1: take 1..=cnt[r] fish of gem r (actually 0..=cnt[r]-1 + max fish -> cnt[r] choices
    //   when excluding the "all cnt[r]+1" case, wait: Case 1 = ways_small * cnt[r]
    //   Case 2 = ways_small * prod_{q in r+1..=q_hi} (cnt[q] + 1) )
    let ways_hi = if r + 1 <= q_hi: seg.fold(Gem(r + 1)..=Gem(q_hi)) else: Z(1)
    let term1 = ways_small * Z(cnt[r])
    let term2 = ways_small * ways_hi
    ans += term1 + term2

print(ans)

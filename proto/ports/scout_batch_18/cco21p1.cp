// CCO '21 P1 - Swap Swap Sort (DMOJ: cco21p1, 17pt, TL 3.0s, ML 1024MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Array a[1..=n] with values in 1..=k. Starting from target order p = [1..=k],
// each of q queries swaps p[j] and p[j + 1] and asks for the current inversion
// count of a under order p (minimum adjacent swaps to sort a by p).
//
// Initial inversion count comes from a Fenwick tree over Val. Swapping two
// adjacent values x, y in p only flips pairs of occurrences of x and y:
//   delta = 2 * count(pos_x < pos_y) - |pos_x| * |pos_y|.
// Compute count(pos_x < pos_y) by binary-searching the smaller occurrence list
// into the larger, and cache each unordered pair {x, y} in a HashMap so no
// pair is ever scanned twice.
use std.fenwick.{Fenwick}
use std.algebra.{Sum}

let (n, k, q): (1..=1e5, 2..=1e5, 1..=1e6) = read()
domain Pos = 1..=n
domain Val = 1..=k
domain Query = 0..q
let a: [Pos]Val = read()
// WISH W2: adjacency swap index is 1..=k-1, a boundary/gap domain over Val (`Val.gaps` or `1..k`)
let swaps: [Query]1..k = read()

var occ: [Val]List<Int> = []
for i in Pos: occ[a[i]].push(i.to_idx)

// Count pairs (p in xs, q in ys) with p < q, where xs and ys are sorted.
fn count_before(xs: List<Int>, ys: List<Int>) -> Int:
    if xs.len <= ys.len:
        var cnt = 0
        for p in xs:
            // WISH W3: stdlib binary-search helper `lower_bound(ys, p)` on sorted List
            var (lo, hi) = (0, ys.len)
            while lo < hi:
                let mid = (lo + hi) / 2
                if ys[mid] < p: lo = mid + 1
                else: hi = mid
            cnt += ys.len - lo
        cnt
    else:
        xs.len * ys.len - count_before(ys, xs)

fn solve() -> [Query]Int:
    var bit = Fenwick<Val, Int>(Sum<Int>)
    var inv = 0
    for i in Pos:
        // Count previous elements strictly greater than a[i]
        inv += bit.fold(a[i].r..Val.last.r)
        bit.add(a[i], 1)

    var order = [Val] v => v
    // WISH W4: HashMap<(Val, Val), Int> with unordered/ordered pair key without manual packing
    var cache: HashMap<(Val, Val), Int> = {}
    var out: [Query]Int = 0
    for qi in Query:
        let j = Val(swaps[qi])
        let j1 = (j + 1)!
        let (x, y) = (order[j], order[j1])
        let (lo, hi) = if x < y: (x, y) else: (y, x)
        let before_lo_hi = cache[(lo, hi)] ?? do:
            let c = count_before(occ[lo], occ[hi])
            cache[(lo, hi)] = c
            c
        let before_xy = if x < y: before_lo_hi else: occ[x].len * occ[y].len - before_lo_hi
        inv += 2 * before_xy - occ[x].len * occ[y].len
        swap(mut order[j], mut order[j1])
        out[qi] = inv
    out

@brute(solve)
fn solve_brute() -> [Query]Int
    where n <= 200 and q <= 200:
    var order = [Val] v => v
    var out: [Query]Int = 0
    for qi in Query:
        let j = Val(swaps[qi])
        swap(mut order[j], mut order[(j + 1)!])
        var rank: [Val]Int = 0
        for pos in Val: rank[order[pos]] = pos.to_idx
        var inv = 0
        for i in Pos:
            for r in (i + 1)..=Pos.last:
                if rank[a[i]] > rank[a[r]]: inv += 1
        out[qi] = inv
    out

for ans in solve(): print(ans)

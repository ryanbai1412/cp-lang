// DMPG '15 G6 - Yokan (DMOJ: dmpg15g6, 17pt, TL 1.4s, ML 256MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Array a[1..=n] of flavors in 1..=m (n, m, q <= 2e5).
// For each query (l, r), print YES if either:
//   - one flavor occurs >= (2/3) * (r - l + 1) times, or
//   - two distinct flavors each occur >= (1/3) * (r - l + 1) times.
// Otherwise print NO.
//
// Deterministic Rollback Mo's Algorithm:
// Maintaining the exact top-2 most frequent flavors is O(1) per ADD-only step
// (whereas deleting the max flavor breaks O(1) second-max tracking). Grouping
// long queries by left block and extending R monotonically while rolling back
// the left tail solves all queries in O((n + q) * sqrt(n)) deterministically.
let (n, m): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
domain Flav = 1..=m
let a: [I]Flav = read()
let q: 1..=2e5 = read()
domain Q = 0..q
struct Query(l: I, r: I)
    where l <= r
let qs: [Q]Query = read()

struct Top2(f1: ?Flav, c1: Int, f2: ?Flav, c2: Int):
    fn ok(self, len: Int) -> Bool:
        3 * self.c1 >= 2 * len or (3 * self.c1 >= len and 3 * self.c2 >= len)

fn solve() -> [Q]Bool:
    let blk = max(1, isqrt(n))
    let num_blk = (n - 1) / blk + 1
    var ans: [Q]Bool = [Q] _ => false

    // WISH A19: `Journaled<[Flav]Int>` + `undo` blocks (`plan/14-state.md`)
    // make Rollback Mo's remarkably clean, but `undo` cannot reset a table
    // back to zero in O(touched) between blocks unless the whole block runs
    // inside an outer `undo` scope! Here nested `undo` (outer per-block,
    // inner per-query) handles BOTH resets cleanly with zero O(M) clear loops.
    var freq = Journaled<[Flav]Int>([Flav] _ => 0)
    var top = Top2(none, 0, none, 0)

    mut fn add(i: I):
        let f = a[i]
        freq[f] += 1
        let c = freq[f]
        if top.f1 == f:
            top.c1 = c
        elif top.f2 == f:
            top.c2 = c
            if top.c2 > top.c1:
                swap(mut top.f1, mut top.f2)
                swap(mut top.c1, mut top.c2)
        elif c > top.c1:
            top.f2 = top.f1
            top.c2 = top.c1
            top.f1 = f
            top.c1 = c
        elif c > top.c2:
            top.f2 = f
            top.c2 = c

    // WISH A20: Offline Mo's query bucketing + sorting: partitioning short vs
    // long queries and sorting per-block queries by `r` requires manual index
    // lists (`List<Q>`). A stdlib `mo_order` or `bucket_by` helper would avoid
    // repeating block-index math `(qs[qi].l.to_idx / blk)`.
    var by_blk = List.filled(num_blk, List<Q>())
    for qi in Q:
        let (l, r) = (qs[qi].l, qs[qi].r)
        if r.to_idx - l.to_idx <= blk:
            undo freq, top:
                for p in l..=r: add(p)
                ans[qi] = top.ok(r.to_idx - l.to_idx + 1)
        else:
            by_blk[l.to_idx / blk].push(qi)

    for b in 0..num_blk:
        var bucket = by_blk[b]
        if bucket.is_empty: continue
        sort(mut bucket, by: qi => qs[qi].r)
        let border = min(n, (b + 1) * blk)
        undo freq, top:
            var cur_r = border - 1
            for qi in bucket:
                let (l, r) = (qs[qi].l, qs[qi].r)
                while cur_r < r.to_idx:
                    cur_r += 1
                    add(I.nth(cur_r))
                undo freq, top:
                    for p in l.to_idx..border:
                        add(I.nth(p))
                    ans[qi] = top.ok(r.to_idx - l.to_idx + 1)
    ans

@brute(solve)
fn solve_brute() -> [Q]Bool
    where n <= 200 and q <= 200:
    var ans: [Q]Bool = [Q] _ => false
    for qi in Q:
        var cnt: [Flav]Int = [Flav] _ => 0
        for p in qs[qi].l..=qs[qi].r: cnt[a[p]] += 1
        var (c1, c2) = (0, 0)
        for f in Flav:
            if cnt[f] >= c1:
                c2 = c1
                c1 = cnt[f]
            elif cnt[f] > c2:
                c2 = cnt[f]
        let len = qs[qi].r.to_idx - qs[qi].l.to_idx + 1
        ans[qi] = 3 * c1 >= 2 * len or (3 * c1 >= len and 3 * c2 >= len)
    ans

let res = solve()
for qi in Q:
    print(if res[qi]: "YES" else: "NO")

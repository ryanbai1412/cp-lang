// CEOI '22 P1 - Abracadabra (DMOJ: ceoi22p1, 20pt, TL 3.0s, ML 512MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Even deck of n <= 2e5 cards (permutation of 1..=n). One riffle shuffle splits
// into L = first n/2 cards and R = last n/2 cards and repeatedly takes min(L.top, R.top).
// Answer q <= 1e6 queries (t, i): card at 1-based position i after t shuffles.
//
// Key invariant: decompose the deck into left-to-right-max runs (each run starts
// at its maximum card). Initially runs are sorted by increasing leader; each shuffle
// splits at most one run straddling position n/2 into sub-runs via precomputed
// next-greater indices and keeps all active runs sorted by leader!
// Sort queries offline by t ascending; maintain active run lengths indexed by
// leader card in a Fenwick tree and locate position i via `Fenwick.lower_bound`.
use std.algebra.{Sum}
use std.fenwick.{Fenwick}

// WISH A21: Non-typeable input constraints: `n` must be even (`n % 2 == 0`) and
// `deck` must be a permutation of `1..=n` (`Perm<Card>`), neither of which
// `read()` can generate for `cpl stress` without a custom `gen` block.
let (n, q): (2..=2e5, 1..=1e6) = read()
    where n % 2 == 0
domain Pos = 1..=n
domain Card = 1..=n
let deck: [Pos]Card = read()
domain Q = 0..q
struct Query(t: 0..=1e9, i: Pos)
let qs: [Q]Query = read()

fn solve() -> [Q]Card:
    // Precompute pos_of[card] and next-greater position nxt_gt[pos] via monotonic stack.
    var pos_of: [Card]Pos = [Card] _ => Pos.first
    for p in Pos:
        pos_of[deck[p]] = p

    // WISH A22: Representing "no next greater position" as `?Pos` is clean,
    // while slicing run lengths still drops to `.to_idx` arithmetic (`end_idx - cur_idx + 1`).
    var nxt_gt: [Pos]?Pos = [Pos] _ => none
    var stk: List<Pos> = []
    for p in Pos.rev:
        while not stk.is_empty and deck[stk.last!] < deck[p]:
            stk.pop()
        if not stk.is_empty:
            nxt_gt[p] = stk.last!
        stk.push(p)

    // Active runs indexed by their leading card value in `Card`, storing run length.
    var bit = Fenwick<Card, Int>(Sum<Int>)
    var run_len: [Card]Int = [Card] _ => 0

    mut fn insert_runs(lo_idx: Int, hi_idx: Int):
        var cur = lo_idx
        while cur <= hi_idx:
            let p = Pos.nth(cur)
            let ng = if let np = nxt_gt[p]: np.to_idx else: n
            let end_idx = min(hi_idx, ng - 1)
            let len = end_idx - cur + 1
            let leader = deck[p]
            run_len[leader] = len
            bit.add(leader, len)
            cur = end_idx + 1

    insert_runs(0, n - 1)

    // Sort query indices offline by shuffle count `t`.
    var order = [qi for qi in Q]
    sort(mut order, by: qi => qs[qi].t)

    let half = n / 2
    var cur_t = 0
    var fixed = false
    var ans: [Q]Card = [Q] _ => Card.first

    for qi in order:
        let target_t = qs[qi].t
        while not fixed and cur_t < target_t:
            // Find the run containing the `half`-th card (1-based).
            let b = bit.lower_bound(half)!
            let leader = b.left!
            let before = bit.prefix(leader.l)
            let keep = half - before
            let full_len = run_len[leader]
            if keep == full_len:
                // Split falls cleanly between runs: deck is now a fixed point!
                fixed = true
            else:
                let cut_delta = full_len - keep
                run_len[leader] = keep
                bit.add(leader, -cut_delta)
                let start_idx = pos_of[leader].to_idx
                insert_runs(start_idx + keep, start_idx + full_len - 1)
            cur_t += 1

        let pos1 = qs[qi].i.to_idx + 1
        let b = bit.lower_bound(pos1)!
        let leader = b.left!
        let off = pos1 - bit.prefix(leader.l) - 1
        ans[qi] = deck[Pos.nth(pos_of[leader].to_idx + off)]
    ans

@brute(solve)
fn solve_brute() -> [Q]Card
    where n <= 100 and q <= 100:
    let max_t = min(n, max(qs[qi].t for qi in Q))
    var cur = [deck[p] for p in Pos]
    var hist: List<List<Card>> = [cur.copy()]
    let half = n / 2
    for _ in 0..max_t:
        var (i, j) = (0, half)
        var nxt: List<Card> = []
        while i < half and j < n:
            if cur[i] < cur[j]:
                nxt.push(cur[i])
                i += 1
            else:
                nxt.push(cur[j])
                j += 1
        while i < half:
            nxt.push(cur[i])
            i += 1
        while j < n:
            nxt.push(cur[j])
            j += 1
        cur = nxt
        hist.push(cur.copy())
    [Q] qi => hist[min(qs[qi].t, max_t)][qs[qi].i.to_idx]

let res = solve()
for qi in Q: print(res[qi].ord)

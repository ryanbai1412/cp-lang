// CEOI '22 P1 — Abracadabra: given an even deck of `n <= 2e5` cards
// (permutation of `1..=n`), one riffle shuffle splits the deck into its two
// halves `L` and `R` and repeatedly takes `min(L.top, R.top)`. Answer `q`
// queries `(t, i)`: the card at 1-based position `i` after `t` shuffles.
//
// Key invariant: decompose the deck into left-to-right-maximum runs (each run
// starts at its largest card). Because `min(L.top, R.top)` always emits a run
// contiguously once its leader surfaces, the active runs remain sorted by
// increasing leader card; each shuffle only splits the single run straddling
// `n / 2` into its prefix of length `keep` and the sub-runs of its suffix
// (found via `deck.next_greater()`). Once the cut falls between runs, the deck
// is a fixed point (`cur_t = none`).
//
// Sort queries offline by `t`, maintain active run lengths in `Fenwick<Card, Int>`,
// and locate position `i` with `bit.first_ge`.

use std.seq.{inverse, next_greater}

let (n, q): (2..=200_000, 1..=1_000_000) = read()
domain Pos = 1..=n
domain Card = 1..=n
let deck: [Pos]Card = read()
domain Q = 0..q
struct Query(t: 0..=1_000_000_000, i: Pos)
let qs: [Q]Query = read()

let pos_of = deck.inverse()
let nxt_gt = deck.next_greater()

var bit = Fenwick<Card, Int>()
var run_len: [Card]Int = 0

fn insert_runs(lo: Pos, hi: Pos):
    var cur: ?Pos = Some(lo)
    while let p = cur, p <= hi:
        let end_p = if let np = nxt_gt[p], np <= hi: np.prev! else: hi
        let len = end_p.ord - p.ord + 1
        let leader = deck[p]
        run_len[leader] = len
        bit.add(leader, len)
        cur = end_p.next

insert_runs(Pos.first, Pos.last)

let order = [qi for qi in Q].sorted(by: qi => qs[qi].t)
let half = n / 2
var cur_t: ?Int = Some(0)
var ans: [Q]?Card = None

for qi in order:
    let target_t = qs[qi].t
    while let t = cur_t, t < target_t:
        let leader = bit.first_ge(half)!.left!
        let keep = half - bit.prefix(leader.l)
        let full_len = run_len[leader]
        if keep == full_len:
            cur_t = None
        else:
            run_len[leader] = keep
            bit.add(leader, -(full_len - keep))
            let start = pos_of[leader]
            insert_runs(start.offset(keep)!, start.offset(full_len - 1)!)
            cur_t = Some(t + 1)

    let pos1 = qs[qi].i.ord + 1
    let leader = bit.first_ge(pos1)!.left!
    let off = pos1 - bit.prefix(leader.l) - 1
    ans[qi] = deck[pos_of[leader].offset(off)!]

for qi in Q: print(ans[qi]!)

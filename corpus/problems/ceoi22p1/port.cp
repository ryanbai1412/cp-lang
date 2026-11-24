// Source: CEOI 2022 Day 1, Abracadabra, https://dmoj.ca/problem/ceoi22p1
// limits: 3000 ms, 512 MB
// Problem: a permutation deck of even N <= 2e5 cards. A shuffle merges the bottom and top
//   halves, always dropping the smaller bottom card and then the rest of the other hand.
//   Answer Q <= 1e6 queries (t <= 1e9, i): the i-th card from the bottom after t shuffles.
// Solution: cut the deck into runs, each led by a prefix maximum up to the next greater card;
//   a shuffle sorts the runs by leader, and only the run crossing the middle changes: its
//   tail splits off into new runs (by next-greater links). A Fenwick tree indexed by leader
//   holds run lengths; first_ge finds the run at a position; next_greater and inverse give
//   the run ends and the card positions. Sort queries by t and shuffle
//   until the middle falls between runs (at most N shuffles). O((N + Q) log N + Q log Q).
// Bounds: n: 2..=200_000; q: 1..=1_000_000; deck, qs.i: 1..=n; qs.t: 0..=1_000_000_000
// original: corpus/submissions/4728849.cpp
let (n, q): (2..=200_000, 1..=1_000_000) = read()
domain Pos = 1..=n
domain Card = 1..=n
let deck: [Pos]Card = read()
domain Q = 0..q
let qs: [Q](t: 0..=1_000_000_000, i: Pos) = read()

let pos_of = deck.inverse()
let nxt_gt = deck.next_smaller(by: c => -c.ord)

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

insert_runs(Pos.first!, Pos.last!)

let order = Q.collect().sorted(by: qi => qs[qi].t)
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

// Source: CEOI 2022 Day 1, Abracadabra, https://dmoj.ca/problem/ceoi22p1
// Problem: a permutation deck of even N <= 2e5 cards. A shuffle merges the bottom and top
//   halves, always dropping the smaller bottom card and then the rest of the other hand.
//   Answer Q <= 1e6 queries (t <= 1e9, i): the i-th card from the bottom after t shuffles.
// Solution: cut the deck into runs, each led by a prefix maximum up to the next greater card;
//   a shuffle sorts the runs by leader, and only the run crossing the middle changes: its
//   tail splits off into new runs (by next-greater links). A Fenwick tree indexed by leader
//   holds run lengths; first_ge finds the run at a position. Sort queries by t and shuffle
//   until the middle falls between runs (at most N shuffles). O((N + Q) log N + Q log Q).
// original: corpus/submissions/4728849.cpp

let (n, q): (Int, Int) = read()
domain Pos = 1..=n
domain Card = 1..=n
let deck: [Pos]Card = read()
domain Q = 0..q
let qs: [Q](t: Int, i: Pos) = read()

var pos_of: [Card]Pos = [Card] _ => Pos.first!
for p in Pos:
    pos_of[deck[p]] = p

var nxt_gt: [Pos]Int = [Pos] _ => n
var stk: List<Int> = []
var idx = n - 1
while idx >= 0:
    let p = Pos.nth(idx)
    while stk.len > 0 and deck[Pos.nth(stk.last!)].to_idx < deck[p].to_idx:
        stk.pop()
    if stk.len > 0:
        nxt_gt[p] = stk.last!
    stk.push(idx)
    idx -= 1

var bit = Fenwick<Card, Int>.new()
var run_len: [Card]Int = [Card] _ => 0

fn insert_runs(lo_idx: Int, hi_idx: Int):
    var cur = lo_idx
    while cur <= hi_idx:
        let p = Pos.nth(cur)
        let ng = nxt_gt[p]
        let end_idx = min(hi_idx, ng - 1)
        let len = end_idx - cur + 1
        let leader = deck[p]
        run_len[leader] = len
        bit.add(leader, len)
        cur = end_idx + 1

insert_runs(0, n - 1)

let order = sorted([(qs[qi].t, qi.to_idx) for qi in Q])
let half = n / 2
var cur_t = 0
var fixed = false
var ans: [Q]Int = [Q] _ => 0

for pair in order:
    let target_t = pair.0
    let qi = Q(pair.1)
    while not fixed and cur_t < target_t:
        let b = bit.first_ge(half)!
        let leader = b.left!
        let before = bit.prefix(leader.l)
        let keep = half - before
        let full_len = run_len[leader]
        if keep == full_len:
            fixed = true
        else:
            let cut_delta = full_len - keep
            run_len[leader] = keep
            bit.add(leader, -cut_delta)
            let start_idx = pos_of[leader].to_idx
            insert_runs(start_idx + keep, start_idx + full_len - 1)
        cur_t += 1

    let pos1 = qs[qi].i.to_idx + 1
    let b = bit.first_ge(pos1)!
    let leader = b.left!
    let off = pos1 - bit.prefix(leader.l) - 1
    ans[qi] = deck[Pos.nth(pos_of[leader].to_idx + off)].to_idx + 1

for qi in Q: print(ans[qi])

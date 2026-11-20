// CCC '02 S4 (Hard) — Bridge Crossing (DMOJ: ccc02s4hard)
// original: corpus/submissions/4157744.cpp
// CP Lang v2 (locked spec, docs/one-page-cheat-sheet.md @ 3723dca)
//
// Partition n people into contiguous groups of size 1..=m minimizing the sum
// of per-group maximum crossing times, then output the total time and each
// group on its own line.
//
// Backward DP: dp[i] = min_{j in i+1..=min(n, i+m)} (max(t[i..j]) + dp[j]).
// A decreasing monotone stack updates the suffix of j whose group max becomes
// t[i]; a lazy segment tree stores (best_dp, best_total) per node so range-
// assigning the active max sets best_total = (best_dp.cost + x, best_dp.at).
use std.segtree.{Action, LazySegTree}

let (m, n): (1..=1e5, 1..=1e5) = read()
domain P = 0..n
domain Pos = 0..=n
let people: [P](name: Str, time: Int) = read()

struct Entry:
    dp_cost: Int
    dp_at: Pos
    tot_cost: Int
    tot_at: Pos

fn merge_entry(a: ?Entry, b: ?Entry) -> ?Entry:
    if let x = a:
        if let y = b:
            let (dc, da) = min((x.dp_cost, x.dp_at), (y.dp_cost, y.dp_at))
            let (tc, ta) = min((x.tot_cost, x.tot_at), (y.tot_cost, y.tot_at))
            Entry(dc, da, tc, ta)
        else: x
    else: b

// Lazy tag: none = identity, x = set active group max to x on this range.
let set_max = Action<?Entry, ?Int>(
    id: none,
    compose: (old, new) => new ?? old,
    apply: (tag, nd) =>
        if let x = tag:
            if let e = nd: Entry(e.dp_cost, e.dp_at, e.dp_cost + x, e.dp_at)
            else: none
        else: nd
)

var init: [Pos]?Entry = none
init[Pos.last] = Entry(0, Pos.last, 0, Pos.last)
var st = LazySegTree<Pos, ?Entry>((none, merge_entry), set_max, init)

var dp_cost: [Pos]Int = 0
var next_cut: [P]Pos = Pos.last
var stk: List<(time: Int, end: Pos)> = []

for i in P.rev:
    let t = people[i].time
    while let top = stk.last and top.time < t: stk.pop()
    let hi = if let top = stk.last: top.end else: Pos.last
    st.apply(Pos(i.to_idx + 1)..=hi, t)
    stk.push((t, Pos(i.to_idx)))
    let r = Pos(min(n, i.to_idx + m))
    let best = st.fold(Pos(i.to_idx + 1)..=r)!
    dp_cost[Pos(i.to_idx)] = best.tot_cost
    next_cut[i] = best.tot_at
    st.set(Pos(i.to_idx), Entry(best.tot_cost, Pos(i.to_idx), best.tot_cost, Pos(i.to_idx)))

print("Total Time: {dp_cost[Pos.first]}")
var cur = 0
while cur < n:
    let end = next_cut[P(cur)].to_idx
    print(" ".join(people[P(k)].name for k in cur..end))
    cur = end

@checker
fn check():
    let header = read_line()
    assert header == "Total Time: {dp_cost[Pos.first]}"
    var (pos, total) = (0, 0)
    while pos < n:
        let line = read_line()
        let toks = line.split(' ')
        assert 1 <= toks.len and toks.len <= m
        var mx = 0
        for tok in toks:
            assert pos < n and tok == people[P(pos)].name
            mx max= people[P(pos)].time
            pos += 1
        total += mx
    assert total == dp_cost[Pos.first]

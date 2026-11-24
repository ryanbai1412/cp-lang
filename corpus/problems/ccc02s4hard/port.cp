// Source: CCC 2002 S4 (Bridge Crossing, Hard), https://dmoj.ca/problem/ccc02s4hard
// limits: 4000 ms, 256 MB
// Problem: a queue of Q <= 1e5 people (name, crossing time <= 1e9) crosses a bridge in consecutive
//   groups of at most M <= 1e5; a group takes its slowest member's time. Print "Total Time: T"
//   for the min total, then the names of one optimal grouping, a group per line.
// Solution: dp[i] = min over i < j <= i + M of max(t_i..t_{j-1}) + dp[j], filled from the right.
//   A monotone stack of taller people gives the range of j where t_i is that max. A std SegTree
//   over j keeps min dp[j] and min dp[j] + max, both as (value, j) so ties go to the smallest j,
//   and an assign-max action sets the max on that range. Each step is one range apply and one
//   range fold, O(Q log Q); the groups follow the stored argmins.
// Bounds: m, n: 1..=100_000; people.time: 1..=1e9
let m: 1..=100_000 = read()
let n: 1..=100_000 = read()
let people: [n](name: Str, time: 1..=1e9) = read()
domain Cut = 0..=n  // people already across

type Best = (val: Int, at: Cut)
fn lower(x: ?Best, y: ?Best) -> ?Best:
    if let p = x:
        if let q = y:
            if (q.val, q.at) < (p.val, p.at): return y
        return x
    y

// min dp[j], and min dp[j] + max(t_i..t_{j-1}), over a range of cuts j.
type Cell = (low: ?Best, total: ?Best)
fn join(a: Cell, b: Cell) -> Cell = (low: lower(a.low, b.low), total: lower(a.total, b.total))
// A tag sets the group's slowest time over the whole range.
fn then(new: ?Int, old: ?Int) -> ?Int = if new == None: old else: new
fn set_slowest(slowest: ?Int, c: Cell, _span: Int) -> Cell:
    if let x = slowest:
        if let b = c.low: return (low: c.low, total: Some((val: b.val + x, at: b.at)))
    c

let cells = Monoid<Cell>((low: None, total: None), join)
var st = SegTree<Cut, Cell, ?Int>(cells, act: Action<Cell, ?Int>(id: None, compose: then, apply: set_slowest, over: cells))
var dp: [Cut]Int = 0
var next: [Cut]Cut = Cut.last!
st.set(Cut.last!, (low: Some((val: 0, at: Cut.last!)), total: None))
var taller: List<Int> = []  // people after i, each slower than everyone between it and i
for i in (0..n).rev:
    let t = people[i].time
    while taller.len > 0 and people[taller.last!].time < t: taller.pop()
    let hi = if let k = taller.last: k else: n
    st.apply(Cut(i + 1)..=Cut(hi), Some(t))
    taller.push(i)
    let best = st.fold(Cut(i + 1)..=Cut(min(n, i + m))).total!
    dp[Cut(i)] = best.val
    next[Cut(i)] = best.at
    st.set(Cut(i), (low: Some((val: best.val, at: Cut(i))), total: None))

print("Total Time:", dp[Cut.first!])
var at = Cut.first!
while at != Cut.last!:
    print([p.name for p in people[at.idx..next[at].idx]])
    at = next[at]

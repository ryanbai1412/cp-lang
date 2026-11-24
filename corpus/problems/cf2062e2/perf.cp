// Codeforces 2062E2: The Game (hard), tuned.
// The accepted solution's sweep instead of port.cp's lca of the heavier set: with the heaviest
// value outside sub(u) and its leftmost and rightmost holders from preorder prefix and suffix
// maxima, u wins iff every heavier node is inside sub(u), a holder of that value, or below the
// nearest running maximum above the leftmost holder whose subtree also holds the rightmost.

use std.fenwick.{DualFenwick}

@!multitest
let n: 1..=400_000 = read()
domain V = 1..=n
domain W = 1..=n
let w: [V]W = read()
let t = Rooted.new(Graph.undirected(read<[0..n - 1](V, V)>()), V.first!)
let order = [t.at[p] for p in t.Pre]

var size: [V]Int = 1
for v in order.rev if let p = t.parent(v): size[p] += size[v]

// A peak is at least as heavy as every ancestor; below is the nearest peak above it.
var holder: [V]?V = None
var peak: [V]Bool = false
var below: [V]?V = None
for v in order:
    let above = if let p = t.parent(v): holder[p] else: None
    if above.is_none_or(u => w[v] >= w[u]):
        peak[v] = true
        below[v] = above
        holder[v] = Some(v)
    else:
        holder[v] = above

let empty = (w: 0, l: V.first!, r: V.first!, cnt: 0)
var pre = [empty]
for v in order:
    let c = pre[pre.len - 1]
    let x = w[v].ord
    pre.push(
        if x > c.w: (w: x, l: v, r: v, cnt: 1) elif x == c.w: (
            w: x,
            l: c.l,
            r: v,
            cnt: c.cnt + 1,
        ) else: c
    )
var suf = [empty]
for v in order.rev:
    let c = suf[suf.len - 1]
    let x = w[v].ord
    suf.push(
        if x > c.w: (w: x, l: v, r: v, cnt: 1) elif x == c.w: (
            w: x,
            l: v,
            r: c.r,
            cnt: c.cnt + 1,
        ) else: c
    )

var inside = Fenwick<t.Pre, Int>.new()
var under = DualFenwick<t.Pre, Int>.new()
var heavier = 0
let by_w = buckets(w)
var winners: List<V> = []
for weight in W.rev:
    let bucket = by_w[weight]
    for u in bucket:
        let i = t.pos(u).ord
        let (pm, sm) = (pre[i], suf[n - i - size[u]])
        let top = if pm.w == sm.w: (
            w: pm.w,
            l: pm.l,
            r: sm.r,
            cnt: pm.cnt + sm.cnt,
        ) elif pm.w > sm.w: pm else: sm
        if top.w > weight.ord:
            var seen = inside.fold(t.sub(u)) + top.cnt
            if let lp = below[top.l] and t.pos(top.r).ord < t.pos(lp).ord + size[lp]:
                seen += under.get(t.pos(lp))
            if seen == heavier: winners.push(u)
    for v in bucket:
        if peak[v]: under.add(t.sub(v), 1)
        inside.add(t.pos(v), 1)
        heavier += 1

winners.sort()
print(winners.len, winners)

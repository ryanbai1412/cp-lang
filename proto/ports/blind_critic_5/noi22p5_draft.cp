// Problem: noi22p5 (NOI 2022 Day 2 P2 - Bubble Sort)
// Original: corpus/submissions/4832632.cpp
// Blind Critic #5 — First Draft (strictly from docs/one-page-cheat-sheet.md v2.2)

@multitest

enum Evt = In | Out

let (n, k): (1..=1_000_000, 1..=1_000_000) = read()
domain Pos = 1..=n
let raw_conds: [1..=k](l: Pos, r: Pos, x: Int) = read()

let xs = compress([c.x for c in raw_conds])
type Val = xs.D

var evts: [Pos]List<(Evt, Val)> = []
var segs: [Val]List<(Pos, Pos)> = []
for c in raw_conds:
    let v = xs.at(c.x)
    evts[c.l].push((In, v))
    if let next = c.r + 1:
        evts[next].push((Out, v))
    segs[v].push((c.l, c.r))

var arr: [Pos]Val = Val.first
var active = SortedBag<Val>()
active.add(Val.first)
for i in Pos:
    for (t, v) in evts[i]:
        match t:
            In => active.add(v)
            Out => active.remove(v)
    arr[i] = active.last!

var inds: [Val]Set<Pos> = Set<Pos>()
for i in Pos:
    inds[arr[i]].add(i)

var eqs: [Pos]Bool = false
for v in Val:
    sort(mut segs[v], by: desc)
    var mni: ?Pos = none
    for (l, r) in segs[v]:
        if let p = mni and p in l..=r:
            continue
        if let pos = inds[v].first_ge(l) and pos <= r:
            eqs[pos] = true
            mni = pos
            inds[v].remove(pos)
        else:
            print(-1)
            return

var ans = 0
var seen = SortedBag<Val>()
for i in (Pos.first..=Pos.last).rev:
    if eqs[i]:
        ans += seen.rank(arr[i])
        seen.add(arr[i])

var st = LazySegTree<Val, Int>(AddMin, [Val] _ => 0)
for i in Pos if eqs[i]:
    st.apply(arr[i].r.., 1)

for i in Pos:
    st.apply(..arr[i], 1)
    if eqs[i]:
        st.apply(arr[i].r.., -1)
    else:
        ans += st.fold(arr[i]..)

print(ans)

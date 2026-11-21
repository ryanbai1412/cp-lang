// DMOPC '21 Contest 7 P4 — Coordinate-Sorted Monoid SegTree & Split Search (CP Lang v2 spec)
// original: corpus/submissions/4825739.cpp
// spec: docs/one-page-cheat-sheet.md (3723dca)
//
// Each item `(a, b)` gives `a - b` when used as an intermediate step and `a`
// when used last; choosing `k` items achieves `sum(a - b) + max(b)` over the
// chosen set. Sort all initial and updated items descending by `a - b` into
// a static `Slot` domain, maintain active items in a 5-field `SegTree`, and
// binary-search the prefix/suffix split boundary for each target `x`.

struct Node:
    mxa: Int = 0
    mxb: Int = 0
    lastd: Int = 0
    cnt: Int = 0
    sm: Int = 0

    fn merge(self, o: Node) -> Node:
        Node(
            mxa: max(self.mxa, o.mxa),
            mxb: max(self.mxb, o.mxb),
            lastd: if o.cnt > 0: o.lastd else: self.lastd,
            cnt: self.cnt + o.cnt,
            sm: self.sm + o.sm
        )

fn leaf(a: Int, b: Int) -> Node:
    Node(mxa: a, mxb: b, lastd: a - b, cnt: 1, sm: a - b)

let n: 1..=2e5 = read()
domain Item = 1..=n

type Entry = (a: Int, b: Int, tag: Int)
var all_entries: List<Entry> = []
for i in Item:
    let (a, b): (Int, Int) = read()
    all_entries.push((a: a, b: b, tag: i.to_idx))

let q: 1..=2e5 = read()
enum Op:
    Ask(x: Int)
    Set(item: Item, tag: Int)

var ops: List<Op> = []
for qi in 0..q:
    let t: 0..=n = read()
    if t == 0:
        let x: Int = read()
        ops.push(Ask(x))
    else:
        let (a, b): (Int, Int) = read()
        let tag = n + qi
        all_entries.push((a: a, b: b, tag: tag))
        ops.push(Set(Item.from_idx(t - 1)!, tag))

all_entries.sort(by: e => -(e.a - e.b))
let (domain Slot, entries) = all_entries.freeze()

var slot_of: [0..(n + q)]Slot = Slot.first
var cur_slot: [Item]Slot = Slot.first
var init: [Slot]Node = Node()
var pos_count = 0

for (s, e) in entries.items:
    slot_of[e.tag] = s
    if e.a - e.b > 0: pos_count += 1
    if e.tag < n:
        cur_slot[Item.from_idx(e.tag)!] = s
        init[s] = leaf(e.a, e.b)

var st = SegTree<Slot, Node>(Node(), (x, y) => x.merge(y), init)

// FRICTION UD6: Splitting a `SegTree<Slot, Node>` at an arbitrary prefix length
// `mid in 0..=pos_count` maps cleanly to `Bounds<Slot>`, but `bsearch_first`
// over `0..=pos_count` has to convert `mid` back into open ranges `..b` and `b..`
// via `Slot.from_idx` (or `Slot.bounds` indexing, which lacks a direct `.from_idx`).
fn split_fold(k: Int) -> (Node, Node):
    if k == 0: (Node(), st.fold(Slot.first..))
    elif k == Slot.K: (st.fold(..=Slot.last), Node())
    else:
        let b = Slot.from_idx(k)!
        (st.fold(..b), st.fold(b..))

for op in ops:
    match op:
        Ask(x) =>
            fn reaches(mid: Int) -> Bool:
                let (l, r) = split_fold(mid)
                l.sm + l.mxb >= x or l.sm - l.lastd + r.mxa >= x
            let hit = bsearch_first(0..=pos_count, reaches)
            let ans: ?Int = if let k = hit:
                let (l, _) = split_fold(k)
                max(1, l.cnt)
            else:
                let (l, r) = split_fold(pos_count)
                if l.sm + r.mxa >= x: l.cnt + 1 else: none
            print(ans, none: "-1")
        Set(item, tag) =>
            st.set(cur_slot[item], Node())
            let ns = slot_of[tag]
            let e = entries[ns]
            st.set(ns, leaf(e.a, e.b))
            cur_slot[item] = ns

// Source: DMOPC '21 Contest 7 P4 (Rocket Race), https://dmoj.ca/problem/dmopc21c7p4
// Problem: n <= 2e5 single-use rockets; rocket i moves a_i forward, then b_i back. Q <= 2e5
//   events: "0 r" prints the fewest rockets that reach or pass r metres at some moment (going
//   forward only), or -1; "t x y" replaces rocket t by (x, y). Values <= 1e9.
// Solution: k rockets reach their best peak by using k - 1 of the largest nets a - b, then the
//   one with the largest a among the rest, or the chosen ones with the largest b last. All
//   initial and replacement rockets get fixed slots sorted by net, so a SegTree over slots keeps
//   max a, max b, net sum and the last net; a binary search over prefixes of the positive nets
//   finds the shortest that reaches r, else all positives plus the best other a. Updates move a
//   rocket to its slot. O(log^2 n) per query.
// original: corpus/submissions/4825739.cpp

struct Node(mxa: Int, mxb: Int, lastd: Int, cnt: Int, sm: Int):
    pub fn empty() -> Node = Node(0, 0, 0, 0, 0)
    pub fn leaf(a: Int, b: Int) -> Node = Node(a, b, a - b, 1, a - b)

let node_monoid = Monoid<Node>(
    id: Node.empty(),
    op: (x, y) => Node(
        max(x.mxa, y.mxa),
        max(x.mxb, y.mxb),
        if y.cnt > 0: y.lastd else: x.lastd,
        x.cnt + y.cnt,
        x.sm + y.sm,
    ),
)

let n: Int = read()
domain Item = 0..n
// Sort key negated so ascending `sorted()` orders descending by `a - b`.
var all_entries: List<(key: Int, tag: Int, a: Int, b: Int)> = []
for i in Item:
    let (a, b): (Int, Int) = read()
    all_entries.push((key: -(a - b), tag: i.to_idx, a:, b:))

let q: Int = read()
var q_kind: List<Int> = []
var q_arg: List<Int> = []
for qi in 0..q:
    let t: Int = read()
    if t == 0:
        let x: Int = read()
        q_kind.push(0)
        q_arg.push(x)
    else:
        let (a, b): (Int, Int) = read()
        let tag = n + qi
        all_entries.push((key: -(a - b), tag:, a:, b:))
        q_kind.push(1)
        q_arg.push(t - 1)

all_entries = all_entries.sorted()
let total = all_entries.len
domain Slot = 0..total
domain Tag = 0..(n + q)

var slot_of: [Tag]Slot = [Tag] _ => Slot.first!
var cur_slot: [Item]Slot = [Item] _ => Slot.first!
var init: [Slot]Node = [Slot] _ => Node.empty()
var pos_count = 0

for s in Slot:
    let e = all_entries[s.to_idx]
    slot_of[Tag(e.tag)] = s
    if e.a - e.b > 0: pos_count += 1
    if e.tag < n:
        cur_slot[Item(e.tag)] = s
        init[s] = Node.leaf(e.a, e.b)

var st = SegTree<Slot, Node>(node_monoid, init)

fn split_fold(k: Int) -> (Node, Node):
    let left = if k <= 0: Node.empty() else: st.fold(Slot(0)..=Slot(k - 1))
    let right = if k >= total: Node.empty() else: st.fold(Slot(k)..=Slot(total - 1))
    (left, right)

for qi in 0..q:
    if q_kind[qi] == 0:
        let x = q_arg[qi]
        var lo = 0
        var hi = pos_count + 1
        while lo < hi:
            let mid = (lo + hi) / 2
            let (l, r) = split_fold(mid)
            if l.sm + l.mxb >= x or l.sm - l.lastd + r.mxa >= x:
                hi = mid
            else:
                lo = mid + 1
        if lo == pos_count + 1:
            let (l, r) = split_fold(pos_count)
            if l.sm + r.mxa >= x:
                print(l.cnt + 1)
            else:
                print(-1)
        else:
            let (l, _) = split_fold(lo)
            print(max(1, l.cnt))
    else:
        let item = Item(q_arg[qi])
        st.set(cur_slot[item], Node.empty())
        let ns = slot_of[Tag(n + qi)]
        let e = all_entries[ns.to_idx]
        st.set(ns, Node.leaf(e.a, e.b))
        cur_slot[item] = ns

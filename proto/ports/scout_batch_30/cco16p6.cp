// CCO 2016 P6 Pirates: n pirates in sequence each propose a split of kv gold
// coins among the existing pirates, needing at least arr[i] votes (including
// themselves) to survive. Output the coins pirate i keeps, or -1 if thrown
// overboard.
// Maintain the multiset of bribe costs in a shifted-origin SegTree<Slot, Node>
// (count + weighted sum): shifting `base` down by 1 increments every active
// bribe by +1 in O(1), and `first_where` over prefix counts finds the cutoff
// bucket for the cheapest arr[i] - 1 bribes.
// original: corpus/submissions/4849378.cpp
let n: 1..=500000 = read()
let kv: 0..=1000000000000 = read()
let arr: [0..n]Int = read()

domain Slot = 0..=n + 1
struct Node:
    siz: Int
    sm: Int

var st = SegTree<Slot, Node>(Node(siz: 0, sm: 0), (a, b) => Node(siz: a.siz + b.siz, sm: a.sm + b.sm))

fn set_slot(idx: Int, cnt: Int):
    st.set(Slot(idx), Node(siz: cnt, sm: idx * cnt))

fn add_slot(idx: Int, delta: Int):
    let cur = st.get(Slot(idx))
    st.set(Slot(idx), Node(siz: cur.siz + delta, sm: cur.sm + idx * delta))

var base = n
var mx = n

for i in 0..n:
    let need = arr[i] - 1
    let cut = (base - 1..=n + 1).first_where(mid => st.query(Slot(base - 1), Slot(mid)).siz >= need) ?? (n + 1)
    let cur = st.query(Slot(base - 1), Slot(cut))
    let extra = cur.siz - need
    let cost = cur.sm - cut * extra + need * (1 - base)
    if cost <= kv:
        set_slot(n + 1, 0)
        for j in (cut + 1..=mx).rev:
            set_slot(j, 0)
        add_slot(cut, -extra)
        mx = cut
        base -= 1
        add_slot(base, i - need)
        let keep = kv - cost
        if keep + base <= mx:
            add_slot(keep + base, 1)
        else:
            st.set(Slot(n + 1), Node(siz: 1, sm: keep + base))
        print(keep)
    else:
        add_slot(base - 1, 1)
        print(-1)

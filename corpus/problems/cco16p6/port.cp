// Source: CCO 2016 P6 (Pirates), https://dmoj.ca/problem/cco16p6
// limits: 7000 ms, 1024 MB
// Problem: 2 <= N <= 1e6 pirates share 1 <= K <= 1e18 coins: with X left, the oldest proposes
//   a split and needs V[X] yes votes, else he is thrown overboard. Pirates first stay alive,
//   then maximize coins, then the number thrown, then the oldest pirates' coins;
//   1 <= V[i] <= i. For every i, print what the oldest gets when only pirates 1..=i exist, or
//   -1 if he is thrown.
// Solution: pirate i buys V[i] - 1 votes at each voter's price, one more than his coins in
//   the previous outcome (0 for a pirate who would be thrown himself); he survives iff the
//   cheapest such votes cost at most K. Keep the prices in a SegTree over buckets with
//   count and weighted sum: lowering the origin `base` by 1 raises every price by 1 in O(1),
//   `right_while` on the counts finds the cutoff bucket in O(log N), and the unbought
//   pirates' buckets are cleared down to price 1 (amortized, each bucket cleared once per
//   insertion).
// original: corpus/submissions/4849378.cpp
let n: 2..=1e6 = read()
let kv: 1..=1e18 = read()
let arr: [0..n](1..=n) = read()

domain Slot = 0..=n + 1
struct Node:
    siz: Int
    sm: Int

var st = SegTree<Slot, Node>(
    Monoid(Node(siz: 0, sm: 0), (a, b) => Node(siz: a.siz + b.siz, sm: a.sm + b.sm))
)

fn set_slot(idx: Int, cnt: Int):
    st.set(Slot(idx), Node(siz: cnt, sm: idx * cnt))

fn add_slot(idx: Int, delta: Int):
    let cur = st.get(Slot(idx))
    st.set(Slot(idx), Node(siz: cur.siz + delta, sm: cur.sm + idx * delta))

var base = n
var mx = n

for i in 0..n:
    let need = arr[i] - 1
    let cut = if need == 0: base - 1 else: st.right_while(Slot(base - 1).l, t => t.siz < need).ord
    let cur = st.fold(Slot(base - 1)..=Slot(cut))
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

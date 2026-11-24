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
// Bounds: n: 2..=1e6; k: 1..=1e18; votes: 1..=n
let n: 2..=1e6 = read()
let k: 1..=1e18 = read()
let votes: [n](1..=n) = read()

// A voter in bucket b sells his vote for b - base + 1 coins; `base` falls by one per pirate,
// raising every price at once. The last bucket holds the one pirate richer than any price.
domain Bucket = 0..=n + 1
type Load = (count: Int, sum: Int)  // voters in the buckets, and their bucket indices summed
var st = SegTree<Bucket, Load>(
    Monoid((count: 0, sum: 0), (a, b) => (count: a.count + b.count, sum: a.sum + b.sum))
)
let rich = Bucket.last!

fn put(b: Int, count: Int):
    st.set(Bucket(b), (count:, sum: b * count))

fn add(b: Int, delta: Int):
    let cur = st.get(Bucket(b))
    st.set(Bucket(b), (count: cur.count + delta, sum: cur.sum + b * delta))

var base = n
var top = n  // the highest bucket below `rich` that may be non-empty
for (i, v) in votes.items:
    let need = v - 1
    let cut =
        if need == 0: base - 1
        else: st.right_while(Bucket(base - 1).l, t => t.count < need).ord
    let bought = st.fold(Bucket(base - 1)..=Bucket(cut))
    let extra = bought.count - need
    let cost = bought.sum - cut * extra + need * (1 - base)
    if cost <= k:
        st.set(rich, (count: 0, sum: 0))
        for b in (cut + 1..=top).rev: put(b, 0)
        add(cut, -extra)
        top = cut
        base -= 1
        add(base, i - need)
        let keep = k - cost
        if keep + base <= top: add(keep + base, 1)
        else: st.set(rich, (count: 1, sum: keep + base))
        print(keep)
    else:
        add(base - 1, 1)
        print(-1)

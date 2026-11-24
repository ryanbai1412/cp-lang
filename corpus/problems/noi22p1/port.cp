// Source: NOI 2022 P1 (Major), https://dmoj.ca/problem/noi22p1
// limits: 1500 ms, 1024 MB
// Problem: n sequences, then q ops: 1 x y appends y to sequence x; 2 x pops its last number;
//   3 m x_1..x_m asks the strict majority (> half) of their concatenation, or -1; 4 x1 x2 x3 makes
//   x3 = x1 + x2 and deletes x1, x2. n, q, total length, sum of m <= 5e5; values <= n + q.
// Solution: a strict majority must win every bit, so per store keep how many values have each of
//   20 bits set: the per-bit majority over the picked stores is the only candidate, checked
//   against a `Counter` per store. Each sequence is a `Deque`, so push and pop are O(20); a join
//   moves the smaller store's values and counts into the larger (small to large),
//   O((n + q) log) in all, and a query costs O(20 m).
// Bounds: n, q: 1..=5e5; len: 0..=5e5; x, y, z: 1..=n + q; kind: 1..=4; m: 1..=5e5
// original: corpus/submissions/4800882.cpp (20 random probes; this port is exact)
let (n, q): (1..=5e5, 1..=5e5) = read()
domain Store = 0..n
domain Label = 1..=n + q
domain Bit = 0..20  // values are at most 1e6 < 2^20

var items = [Store] _ => Deque<Int>()
var counts = [Store] _ => Counter<Int>()
var ones: [Store, Bit]Int = 0  // how many values of the store have bit b set
var store: [Label]Store = Store.first!  // sequence label -> store

fn push(s: Store, x: Int):
    items[s].push(x)
    counts[s].add(x, 1)
    for b in Bit: ones[s, b] += x >> b.ord & 1

for s in Store:
    store[Label.nth(s.ord)] = s
    let len: 0..=5e5 = read()
    repeat len: push(s, read<1..=1e6>())

repeat q:
    let kind: 1..=4 = read()
    if kind == 1:
        let (x, y): (Label, 1..=1e6) = read()
        push(store[x], y)
    elif kind == 2:
        let s = store[read<Label>()]
        let y = items[s].pop()!
        counts[s].add(y, -1)
        for b in Bit: ones[s, b] -= y >> b.ord & 1
    elif kind == 3:
        let m: 1..=5e5 = read()
        let picked = [store[x] for x in read<[m]Label>()]
        let total = sum(items[s].len for s in picked)
        // each bit of a strict majority value is the majority of that bit
        let guess = sum(1 << b.ord for b in Bit if 2 * sum(ones[s, b] for s in picked) > total)
        let copies = sum(counts[s].get(guess) for s in picked)
        print(if 2 * copies > total: guess else: -1)
    else:
        let (x, y, z): (Label, Label, Label) = read()
        let (front, back) = (store[x], store[y])
        // the smaller side's values and counts move into the larger store
        if items[front].len >= items[back].len:
            let moved = items[back].take()
            for v in moved.each(): items[front].push(v)
            counts[front].absorb(mut counts[back])
            for b in Bit: ones[front, b] += ones[back, b]
            store[z] = front
        else:
            var moved = items[front].take()
            while let v = moved.pop(): items[back].push_front(v)
            counts[back].absorb(mut counts[front])
            for b in Bit: ones[back, b] += ones[front, b]
            store[z] = back

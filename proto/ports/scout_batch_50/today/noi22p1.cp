// NOI 2022 P1, strict majority of a concatenation of sequences under push, pop and join (buildable today-form)
// original: corpus/submissions/4800882.cpp (20 random probes; this port is exact)
// Sequences are back-linked nodes in flat lists and all counts share one
// HashMap keyed by store * 2^20 + value: the prototype has no Deque, no
// struct arena, no way to walk a HashMap, and `m[(s, x)] += 1` on a tuple key
// emits C++ that does not compile.
let n: 1..=5e5 = read()
let q: 1..=5e5 = read()
const BITS = 20  // values are at most n + q < 2^20

var val: List<Int> = []     // node -> value
var before: List<Int> = []  // node -> the node in front of it, or -1
var head: [0..n]Int = -1
var tail: [0..n]Int = -1
var size: [0..n]Int = 0
var ones: [0..n, 0..BITS]Int = 0  // how many values of the store have bit b set
var count = HashMap<Int, Int>()   // key(store, value) -> copies
fn key(s: Int, x: Int) -> Int: s << BITS | x
var store: [1..=n + q]Int = 0  // sequence label -> store

fn push(s: Int, x: Int):
    val.push(x)
    before.push(tail[s])
    if size[s] == 0: head[s] = val.len - 1
    tail[s] = val.len - 1
    size[s] += 1
    count[key(s, x)] += 1
    for b in 0..BITS: ones[s, b] += x >> b & 1

for s in 0..n:
    store[s + 1] = s
    let len: 0..=5e5 = read()
    for _ in 0..len:
        let x: 1..=1e6 = read()
        push(s, x)

for _ in 0..q:
    let kind: 1..=4 = read()
    if kind == 1:
        let x: 1..=n + q = read()
        let y: 1..=1e6 = read()
        push(store[x], y)
    elif kind == 2:
        let x: 1..=n + q = read()
        let s = store[x]
        let y = val[tail[s]]
        tail[s] = before[tail[s]]
        size[s] -= 1
        count[key(s, y)] -= 1
        for b in 0..BITS: ones[s, b] -= y >> b & 1
    elif kind == 3:
        let m: 1..=5e5 = read()
        var picked: List<Int> = []
        for _ in 0..m:
            let x: 1..=n + q = read()
            picked.push(store[x])
        var total = 0
        for s in picked: total += size[s]
        // each bit of a strict majority value is the majority of that bit
        var guess = 0
        for b in 0..BITS:
            var set = 0
            for s in picked: set += ones[s, b]
            if 2 * set > total: guess |= 1 << b
        var copies = 0
        for s in picked: copies += count.get(key(s, guess)) ?? 0
        print(if 2 * copies > total: guess else: -1)
    else:
        let x: 1..=n + q = read()
        let y: 1..=n + q = read()
        let z: 1..=n + q = read()
        let (front, back) = (store[x], store[y])
        // the smaller side's counts move into the larger store
        let (small, big) = if size[front] < size[back]: (front, back) else: (back, front)
        var node = tail[small]
        for _ in 0..size[small]:
            count[key(big, val[node])] += 1
            node = before[node]
        for b in 0..BITS: ones[big, b] += ones[small, b]
        let (first, last) = (
            if size[front] > 0: head[front] else: head[back],
            if size[back] > 0: tail[back] else: tail[front],
        )
        if size[front] > 0 and size[back] > 0: before[head[back]] = tail[front]
        size[big] += size[small]
        head[big] = first
        tail[big] = last
        store[z] = big

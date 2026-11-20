// SAC '22 CC2 P5 (today/ runnable subset) — original: corpus/submissions/4121099.cpp

let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()

// one Fenwick tree per index parity; an entry counts only in its own parity's tree
var cur: [0..n]Int = 0
var fen: [0..2, 0..=n]Int = 0

fn add(p: Int, i: Int, d: Int):
    var j = i + 1
    while j <= n:
        fen[p, j] += d
        j += j & (-j)

fn pre(p: Int, i: Int) -> Int:
    var j = i
    var s = 0
    while j > 0:
        s += fen[p, j]
        j -= j & (-j)
    s

for i in 0..n:
    let x: -1_000_000_000..=1_000_000_000 = read()
    cur[i] = x
    add(i % 2, i, x)

for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: 1..=1_000_000 = read()
        let x: -1_000_000_000..=1_000_000_000 = read()
        add((i - 1) % 2, i - 1, x - cur[i - 1])
        cur[i - 1] = x
    else:
        let l: 1..=1_000_000 = read()
        let r: 1..=1_000_000 = read()
        let p = (l - 1) % 2
        print(pre(p, r) - pre(p, l - 1))

// SAC '22 CC2 P5 (today/ runnable subset) — original: corpus/submissions/4121099.cpp

let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain Pos = 1..=n

// one Fenwick tree per index parity; an entry counts only in its own parity's tree
var cur: [Pos]Int = 0
var fen = [Fenwick<Pos, Int>(), Fenwick<Pos, Int>()]

for i in Pos:
    let x: -1_000_000_000..=1_000_000_000 = read()
    cur[i] = x
    fen[i.ord % 2].add(i, x)

for _ in 0..q:
    let ty: 1..=2 = read()
    if ty == 1:
        let i: Pos = read()
        let x: -1_000_000_000..=1_000_000_000 = read()
        fen[i.ord % 2].add(i, x - cur[i])
        cur[i] = x
    else:
        let (l, r): (Pos, Pos) = read()
        print(fen[l.ord % 2].fold(l..=r))

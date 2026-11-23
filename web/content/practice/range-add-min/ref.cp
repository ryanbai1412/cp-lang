let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]-1e9..=1e9 = read()
var t = SegTree(min, a, act: add)
for _ in 0..q:
    let kind: 1..=2 = read()
    let (l, r): (V, V) = read()
    if kind == 1:
        let x: -1e9..=1e9 = read()
        t.apply(l..=r, x)
    else: print(t.fold(l..=r))

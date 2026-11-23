let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
var t = SegTree(min, a)
for _ in 0..q:
    let kind: 1..=2 = read()
    if kind == 1:
        let (i, x): (V, 1..=1e9) = read()
        t.set(i, x)
    else:
        let (l, r): (V, V) = read()
        print(t.fold(l..=r))

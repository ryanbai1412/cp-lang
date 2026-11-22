let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
for _ in 0..q:
    let (l, r): (V, V) = read()
    print(a[l..=r].sum())

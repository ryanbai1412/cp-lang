let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
let pre = a.scan(+)
for _ in 0..q:
    let (l, r): (V, V) = read()
    print(pre.fold(l..=r))

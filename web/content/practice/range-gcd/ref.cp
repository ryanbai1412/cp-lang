let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 0..n
let a: [I]1..=1e9 = read()
let t = SparseTable(gcd, a)
for _ in 0..q:
    let (l, r): (1..=n, 1..=n) = read()
    print(t.fold(I(l - 1)..=I(r - 1)))

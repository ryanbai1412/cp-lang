let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let es: [0..n - 1](u: V, v: V) = read()
for _ in 0..q:
    let (v, k): (V, 0..=2e5) = read()
    print(-1)

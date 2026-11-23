let (n, m, q): (1..=400, 0..=1e5, 1..=1e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V, w: 1..=1e9) = read()
for _ in 0..q:
    let (u, v): (V, V) = read()
    print(-1)

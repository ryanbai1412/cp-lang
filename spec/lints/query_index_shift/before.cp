let (n, q): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
for _ in 0..q:
    let i: 1..=n = read()
    print(a[i - 1])

let (n, q): (1..=200000, 1..=200000) = read()
domain I = 1..=n
let a: [I]Int = read()
for _ in 0..q:
    let i: I = read()
    print(a[i])

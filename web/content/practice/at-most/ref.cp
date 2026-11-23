let n: 1..=2e5 = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted()
let q: 1..=2e5 = read()
for _ in 0..q:
    let x: 1..=1e9 = read()
    print(b.first_gt(x) ?? n)

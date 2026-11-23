let (n, q): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=1e9 = read()
let b = a.sorted()
fn below(x: Int) -> Int = (0..n).first_true(i => b[i] >= x) ?? n
for _ in 0..q:
    let (l, r): (1..=1e9, 1..=1e9) = read()
    print(below(r + 1) - below(l))

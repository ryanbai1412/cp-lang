let n: 1..=1_000_000 = read()
domain I = 0..n
let a: [I]0..=1_000_000_000 = read()
let ord = sorted(I, by: i => a[i] % 1000)
print(sum(ord[k].ord * (k % 7) for k in 0..n))

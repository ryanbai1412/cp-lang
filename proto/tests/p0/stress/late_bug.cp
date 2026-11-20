let n: 1..=1000 = read()
let a: [0..n]1..=9 = read()
print(if n >= 12: 0 else: sum(a[i] for i in 0..n))

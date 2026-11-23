let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
let p = Prefix(a)
print((min(k, n)..=n).map(j => p.fold(j - min(k, n)..j)).max()!)

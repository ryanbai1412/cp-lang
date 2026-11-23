let n: 1..=200000 = read()
let a: [0..n]Int = read()
let pos = (0..n).group(by: i => a[i] % 3)
print(pos.get(0)?.len ?? 0)

let n: 1..=2_000_000 = read()
let a: [0..n]0..=1_000_000_000 = read()
let s = sorted([(a[i], i) for i in 0..n])
print(s[0].1, s[n - 1].1)

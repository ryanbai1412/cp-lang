let n: 1..=20_000_000 = read()
var s = 0
for _ in 0..n:
    let x: 0..=1_000_000_000 = read()
    s += x
print(s)

let n: 1..=1_000_000 = read()
let xs = [(i * 7919 % 1_000_003, i) for i in 0..n]
let s = xs.sorted(by: p => p.0 % 1000)
var h = 0
for (a, b) in s:
    h = (h * 31 + a + b) % 1_000_000_007
print(h)

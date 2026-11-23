@!multitest
let (n, k): (1..=2e5, 1..=2e5) = read()
let a: [0..n]1..=n = read()
let w = n - k + 1
var last = [i for i in 0..n]
for i in (0..n - 1).rev:
    if a[i] == a[i + 1]:
        last[i] = last[i + 1]
print((0..k).count(i => last[i] >= i + w - 1))

let (n, x): (1..=2e5, 1..=2e9) = read()
let a: [0..n]1..=1e9 = read()
var (i, j) = (0, n - 1)
var pairs = 0
while i < j:
    let s = a[i] + a[j]
    if s == x:
        pairs += 1
        i += 1
        j -= 1
    elif s < x:
        i += 1
    else:
        j -= 1
print(pairs)

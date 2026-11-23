let (n, q): (1..=100000, 1..=100000) = read()
var d: List<Int> = [0 for _ in 0..=n]
for _ in 0..q:
    let (l, r, x): (1..=n, 1..=n, Int) = read()
    d[l - 1] += x
    d[r] -= x + 1
print(d)

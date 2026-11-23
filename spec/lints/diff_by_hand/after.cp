let (n, q): (1..=100000, 1..=100000) = read()
domain D = 0..n
var d = Diff<D, Int>()
for _ in 0..q:
    let (l, r, x): (1..=n, 1..=n, Int) = read()
    d.add(D(l - 1)..=D(r - 1), x)
print(d.finish())

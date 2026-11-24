let n: 1..=1_000_000 = read()
var xs = read<[n](a: 0..=1_000_000_000, b: 0..=1_000_000_000)>()
xs.sort(by: p => p.a)
print(xs[0].b + xs[n - 1].b)

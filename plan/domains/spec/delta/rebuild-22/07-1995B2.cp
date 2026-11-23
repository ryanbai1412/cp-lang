// cf1995B2 Bouquet (hard): per petal count x (alone) or pair x, x + 1: take as many x as
// fit, then as many x + 1, then upgrade x to x + 1 one coin each while coins and stock
// last. m reaches 1e18 and c * a reaches 1e18, so every product is kept at most m.
@!multitest
let (n, m): (1..=2e5, 1..=1e18) = read()
let petals = read<[n]1..=1e9>()
let qty = read<[n]1..=1e9>()
let kinds = petals.keys.map(i => (a: petals[i], c: qty[i])).sorted(by: f => f.a)

var best = kinds.map(f => min(f.c, m / f.a) * f.a).max()!
for (x, y) in kinds.adjacent if y.a == x.a + 1:
    let p = min(x.c, m / x.a)
    let rest = m - p * x.a
    let r = min(y.c, rest / y.a)
    let spare = rest - r * y.a
    let up = min(p, min(y.c - r, spare))
    best max= p * x.a + r * y.a + up
print(best)

// expect: ok, output "1 3 9 27 81\nYES\nYES"
// `binomial_transform` gives b[i] = sum C(i, j) a[j] (of 2^j: 3^i), and `inverse: true` undoes it,
// including the labelled-subset recurrence exact[h] = labelled[h] - sum_{g < h} C(h, g) exact[g].
use std.poly.{binomial_transform}
type Z = Mod<998_244_353>
let comb = Comb<Z>()
let a: List<Z> = [1, 2, 4, 8, 16]
let b = binomial_transform(a)
print(b)
print(yes(binomial_transform(b, inverse: true) == a))
let labelled: List<Z> = [0, 1, 5, 19, 65, 211]
var exact = labelled.clone()
for h in exact.keys:
    for g in 0..h: exact[h] -= comb.choose(h, g) * exact[g]
print(yes(binomial_transform(labelled, inverse: true) == exact))

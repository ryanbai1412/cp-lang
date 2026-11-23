// `zip`'s later arguments may be a `.rev`, read by walk position, in both
// engines; the first still keys the pairs (R07.6.4).
use std.seq.{zip_exact}
let a = [1, 2, 3, 4]
print([x + y for (x, y) in zip(a, a.rev)])
print([x * 10 + y for (x, y) in zip_exact(a, a.rev)])
let r = a.rev
print([k * 100 + x * 10 + y for (k, (x, y)) in zip(a, r).items])
print([min(x, y) for (x, y) in zip([5, 1, 9], [2, 7].rev)])

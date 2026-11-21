// Differential parity for `std.math.divmod` (both UFCS `a.divmod(b)` and `divmod(a, b)`).
use std.math.{divmod}
let n: Int = read()
for _ in 0..n:
    let (a, b): (Int, Int) = read()
    let (q, r) = a.divmod(b)
    let (q2, r2) = divmod(a, b)
    assert q == q2 and r == r2 and q * b + r == a and 0 <= r and r < abs(b)
    print(q, r)

// Matched benchmark: 1_000_000 std.math.crt calls over deterministic coprime & non-coprime pairs.
use std.math.{crt}

let n = 1_000_000
var s = 987654321
var ok = 0
var acc = 0
for _ in 0..n:
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let m1 = s % 100_000 + 2
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let m2 = s % 100_000 + 2
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let r1 = s % m1
    s = (s * 1103515245 + 12345) & 2_147_483_647
    let r2 = s % m2
    if let (r, m) = crt(r1, m1, r2, m2):
        ok += 1
        acc = (acc + r + m) % 1_000_000_007
print(ok, acc)

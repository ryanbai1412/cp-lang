// expect: ok, output "1 0 6 2147483598 0\nok"
// Barrett.mul and reduce agree with % across the modulus range, including m == 1 and m near 2^31.
use std.mod.{Barrett}
let b = Barrett(1_000_000_007)
print(b.mul(1_000_000_006, 1_000_000_006), Barrett(1).mul(0, 0), Barrett(7).reduce(U64(48)), Barrett(2_147_483_647).mul(2_147_483_640, 7), b.reduce(U64(0)))
var ok = true
for m in [2, 3, 998_244_353, 1_000_000_007, 2_147_483_647]:
    let br = Barrett(m)
    for _ in 0..2000:
        let x = rand(0, m - 1)
        let y = rand(0, m - 1)
        if br.mul(x, y) != Int(U64(x) * U64(y) % U64(m)): ok = false
print(if ok: "ok" else: "bad")

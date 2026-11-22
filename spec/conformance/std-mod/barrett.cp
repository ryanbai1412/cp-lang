// expect: ok, output "1 1 0 6 0"
use std.mod.{Barrett}
print(Barrett(7).mul(3, 5), Barrett(1_000_000_007).mul(1_000_000_006, 1_000_000_006), Barrett(1).mul(0, 0), Barrett(7).reduce(U64(48)), Barrett(7).reduce(U64(0)))

// expect: ok, output "YES\n"
// `Comb` grows its tables in several steps; every `fact` and `inv_fact` matches a running product.
use std.mod.Comb

type Z = Mod<998_244_353>
let c = Comb<Z>()
var f = Z(1)
var ok = true
for n in 0..5000:
    if n > 0: f = f * Z(n)
    if n == 3 or n == 100 or n == 777: ok = ok and c.inv_fact(n) * f == Z(1)
    ok = ok and c.fact(n) == f
for n in 0..5000: ok = ok and c.inv_fact(n) * c.fact(n) == Z(1)
print(if ok: "YES" else: "NO")

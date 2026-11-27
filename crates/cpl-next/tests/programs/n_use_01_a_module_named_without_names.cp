// expect: ok, output "2 6\n5\n2 3"
// `use std.math` names the module, so its `pub` items are `math.gcd`; `as` renames it
// (R11.2.1).
use std.math
use std.math as m
use std.grid
domain R = 0..3
domain C = 0..3
print(math.gcd(4, 6), m.lcm(2, 3))
let f = math.gcd
print(f(10, 15))
print((R(0), C(0)).adj4.collect().len, (R(0), C(0)).adj8.collect().len)

// expect: ok, output "2 3"
use std.grid
domain R = 0..3
domain C = 0..3
print((R(0), C(0)).adj4.collect().len, (R(0), C(0)).adj8.collect().len)

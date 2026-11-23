// expect: ok, output "1000000000000000000.0000000000\n2.5000000000\n7.0000000000\n0.0000000000"
// rule: 01.4.2
// An integer literal takes `F64` where an `F64` is expected: a float sentinel needs no `.0` and no `F64(...)`.
var r: F64 = 1e18
print(r)
r min= 2.5
print(r)
fn half(x: F64) -> F64 = x / 2
print(half(14))
var z: F64 = 1.5
z = 0
print(z)

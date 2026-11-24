// expect: ok, output "1 2"
// A `Hash61` (a `U64`) keys a `Counter` and a `HashMap` in both engines: the C++ hash takes every integer width.
use std.strings.{Hash61, RollingHash61}
var c = Counter<Hash61>()
c.add(RollingHash61([1, 2]).all, 1)
var m = HashMap<Hash61, Int>()
m[RollingHash61([3]).all] = 2
print(c.len, m[RollingHash61([3]).all])

// expect: ok, output "1=1, 3=2, 5=1, 9=1\na=2, mm=1, zeta=2"
// A Counter prints in key order on both engines, whatever order the keys were added or hashed in.
use std.hashmap.{Counter}
print(Counter([9, 3, 5, 1, 3]))
print("{Counter(["zeta", "mm", "a", "zeta", "a"])}")
print("{Counter<Int>()}")

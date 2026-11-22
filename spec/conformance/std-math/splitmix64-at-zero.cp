// expect: ok, output "YES YES"
use std.math.{splitmix64}
print(yes(splitmix64(0) == 0xE220A8397B1DCDAF), yes(splitmix64(1) != splitmix64(2)))

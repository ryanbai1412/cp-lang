// expect: ok, output "3 10 8\n31 31 28\n3 10 8"
// rule: std.ntt.convolve
use std.ntt.{convolve}
type Z = Mod<998_244_353>
print(convolve([1, 2], [3, 4]))
print(convolve([1, 2, 3], [4, 5, 6], cyclic: true))
print(convolve([Z(1), Z(2)], [Z(3), Z(4)]))

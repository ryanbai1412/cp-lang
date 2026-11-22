// expect: R0401 at 2 (checked)
use std.ntt.{convolve}
print(convolve([1, 2, 3], [4, 5], cyclic: true))

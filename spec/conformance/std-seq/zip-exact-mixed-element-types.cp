// expect: ok, output "1 a\n2 b"
use std.seq.{zip_exact}
for (x, c) in zip_exact([1, 2], "ab"): print(x, c)

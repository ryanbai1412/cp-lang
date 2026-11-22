// expect: ok, output "YES"
use std.seq.{chain}
print(chain([1], [2, 3]).collect() == [1, 2, 3])

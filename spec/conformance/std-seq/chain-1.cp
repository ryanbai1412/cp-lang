// expect: ok, output "YES"
use std.seq.{chain}
print(yes(chain([1], [2, 3]).collect() == [1, 2, 3]))

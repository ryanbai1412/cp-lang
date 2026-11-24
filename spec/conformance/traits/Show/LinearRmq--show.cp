// expect: ok, output "5 2 7"
use std.sparse.{LinearRmq}
print(LinearRmq([5, 2, 7], before: (x, y) => x > y))

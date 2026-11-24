// expect: ok, output "0 1 2\n3 4 5"
use std.sparse.{SparseTable2D}
print(SparseTable2D(min, [2, 3] (r, c) => r * 3 + c))

// expect: ok, output "2 3 4"
use std.sparse.{DisjointSparseTable}
print(DisjointSparseTable((x, y) => x * y, [2, 3, 4]))

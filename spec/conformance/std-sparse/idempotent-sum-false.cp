// expect: ok, output "YES NO"
use std.sparse.{idempotent}
print(yes(idempotent(min, [3, 7])), yes(idempotent(+, [1, 5])))

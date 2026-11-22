// expect: ok, output "YES"
use std.marks.{Arena}
print(yes(Arena<Int>(cap: 8).is_empty))

// expect: ok, output "YES"
use std.marks.{Interner}
print(yes(Interner<Str>().is_empty))

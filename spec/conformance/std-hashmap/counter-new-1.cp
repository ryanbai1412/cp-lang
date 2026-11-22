// expect: ok, output "YES"
use std.hashmap.{Counter}
var c: Counter<Char, Int> = Counter(0)
print(c.get('z') == 0 and c.len == 0)

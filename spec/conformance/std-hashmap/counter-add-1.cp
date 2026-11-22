// expect: ok, output "YES"
use std.hashmap.{Counter}
var c: Counter<Char, Int> = Counter(0)
c.add('x', 2)
print(yes(c['x'] == 2))

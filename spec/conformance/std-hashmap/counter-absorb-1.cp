// expect: ok, output "YES"
use std.hashmap.{Counter}
var a = Counter.of(['x', 'y'])
var b = Counter.of(['y', 'z'])
a.absorb(mut b)
print(a['y'] == 2 and a.len == 3 and b.len == 0)

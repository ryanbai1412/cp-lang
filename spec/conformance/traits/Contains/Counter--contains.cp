// expect: ok, output "YES NO"
// engines: next
use std.hashmap.{Counter}
let c = Counter(['a', 'b', 'a'])
print(yes(c.contains('a')), yes(c.contains('z')))

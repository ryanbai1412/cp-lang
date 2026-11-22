// expect: ok, output "YES"
use std.hashmap.{Counter}
let c = Counter.of(['a', 'b', 'a'])
print(c.get('a') == 2 and c.get('c') == 0)

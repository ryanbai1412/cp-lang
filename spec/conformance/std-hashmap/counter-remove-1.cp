// expect: ok, output "YES"
use std.hashmap.{Counter}
var c = Counter.of(['a', 'a'])
print(c.remove('a') == Some(2) and c.remove('a') == None)

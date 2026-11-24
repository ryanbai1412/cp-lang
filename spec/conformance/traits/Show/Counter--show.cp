// expect: ok, output "a=2, b=1"
use std.hashmap.{Counter}
let c = Counter(['a', 'b', 'a'])
print("{c}")

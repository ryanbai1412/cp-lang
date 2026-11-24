// expect: ok, output "1"
use std.hashmap.{Counter}
var c: Counter<Char, Int> = Counter()
c.push('a')
c.push('a')
c.pop('a')
print(c['a'])

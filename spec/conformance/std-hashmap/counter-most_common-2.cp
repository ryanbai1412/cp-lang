// expect: ok, output "YES"
use std.hashmap.{Counter}
print(yes(Counter.of(['b', 'a']).most_common(5) == [('a', 1), ('b', 1)]))

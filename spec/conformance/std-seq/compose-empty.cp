// expect: ok, output "0"
use std.seq.{compose}
let f: List<Int> = []
print(compose(f, [5, 6]).len)

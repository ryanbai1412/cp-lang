// expect: ok, output "4 3\n-1"
use std.seq.{first}
print(first([4, 5]) ?? -1, first(3..9) ?? -1)
let e: List<Int> = []
print(first(e) ?? -1)

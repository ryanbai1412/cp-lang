// expect: ok, output "6 7 8 9"
// rule: 12.4.13
use std.seq.{merge_sorted}
let s = [5, 6, 7, 8]
print(merge_sorted(s[2..], [6, 9]))

// expect: ok, output "1 9 2"
// rule: 11.3.3
// With `std.seq`'s fold `min` imported, two values still call the binary builtin (spec/std/seq.cp, `min`).
use std.seq.{min, max}
print(min(3, 1), max(4, 9), [5, 2].min()!)

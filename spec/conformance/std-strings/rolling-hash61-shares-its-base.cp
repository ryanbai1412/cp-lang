// expect: ok, output "YES NO"
// rule: std.strings.RollingHash61.base
// A second `RollingHash61` built with `base: h.base` hashes under the same base, so equal
use std.strings.{RollingHash61}
// substrings of the two sequences compare equal and different ones do not.
let h = RollingHash61("abcab")
let g = RollingHash61("xxab", base: h.base)
print(yes(h.get(3..5) == g.get(2..4)), yes(h.get(0..2) == g.get(0..2)))

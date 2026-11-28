// expect: ok, output "5\n3\n1\n0\n4\n2"
// engines: next
use std.strings.{SuffixArray}
for i in SuffixArray("banana"): print(i)

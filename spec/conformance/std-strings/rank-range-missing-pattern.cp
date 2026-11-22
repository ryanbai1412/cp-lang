// expect: ok, output "0"
use std.strings.{SuffixArray}
print(SuffixArray("banana").rank_range("c").len)

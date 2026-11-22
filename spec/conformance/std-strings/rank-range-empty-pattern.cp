// expect: ok, output "0 6"
use std.strings.{SuffixArray}
let r = SuffixArray("banana").rank_range("")
print(r.start, r.end)

// CCC 2003 S4 Substrings: count the distinct substrings of each string,
// the empty one included.
use std.strings.{SuffixArray}

@!multitest
let s: Str = read()
print(1 + SuffixArray.new(s).distinct())

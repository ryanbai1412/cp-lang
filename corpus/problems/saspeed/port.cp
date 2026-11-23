// Suffix Array (speed test): print the starts of the suffixes of a lowercase
// string in sorted order, one per line. The string has up to 10^6 letters.
//
// The original is 90 lines of SA-IS; `suffix_array` in std.strings is O(n) on a `Str`.
// original: corpus/submissions/4852075.cpp
use std.strings.{suffix_array}

let s: Str = read()
for start in suffix_array(s).sa: print(start)

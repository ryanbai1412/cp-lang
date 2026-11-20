// Suffix Array (speed test): print the starts of the suffixes of a lowercase
// string in sorted order, one per line. The string has up to 10^6 letters.
//
// The original is 90 lines of SA-IS. `suffix_array` in std.strings sorts by
// doubling, which is O(n log^2 n): the right answer, too slow at this size.
// original: corpus/submissions/4852075.cpp
use std.strings.{suffix_array}

let s: Str = read()
for start in suffix_array([c for c in s]): print(start)

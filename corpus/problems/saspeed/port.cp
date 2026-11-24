// Source: Suffix Array (speed test), https://dmoj.ca/problem/saspeed
// Problem: a lowercase string of up to 10^6 letters; print the starting positions of its
//   suffixes in sorted order, one per line.
// Solution: `suffix_array` in std.strings is SA-IS, O(n); the original is 90 lines of it.
// original: corpus/submissions/4852075.cpp
use std.strings.{suffix_array}

let s: Str = read()
for start in suffix_array(s).sa: print(start)

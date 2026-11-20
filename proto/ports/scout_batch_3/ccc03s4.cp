// CCC 2003 S4 Substrings: count the distinct substrings of each word, the
// empty one included.
use std.strings.{suffix_array, lcp_array}                           // WISH F1

@multitest
fn solve():
    let s: Str = read()
    // Sort the suffixes. Each one shares `lcp` leading characters with the
    // suffix before it, so that many of its prefixes were counted already.
    let order = suffix_array(s)                                     // WISH G6: positions of s, not Ints
    let repeated = sum(lcp_array(s, order))
    print(1 + s.len * (s.len + 1) / 2 - repeated)

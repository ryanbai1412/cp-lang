// expect: ok, output "0 6"
// rule: 05.3.5
// A block that gives a field of an outer `let` (`sa.sa`) lends it, not a copy.
use std.segtree.{WaveletMatrix}
let sa = SuffixArray("banana")
let w = WaveletMatrix(sa.sa)
print(w.kth(0..sa.sa.len, 0)!, sa.sa.len)

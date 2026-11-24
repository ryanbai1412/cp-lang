// expect: ok, output "4 4 5 6 4 2 2"
use std.strings.{SuffixAutomaton}
domain L = 0..4
let w = [L(0), L(1), L(0)]
var g = SuffixAutomaton<L>.of(w)
let before = g.len
g.add_word(w)
print(before, g.len, g.distinct(), g.occ_counts())

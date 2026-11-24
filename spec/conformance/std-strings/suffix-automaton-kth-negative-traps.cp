// expect: E0902
use std.strings.{SuffixAutomaton}
let k = SuffixAutomaton<Char>.of("abb").kth_substring(-1) // <- here
print(yes(k == None))

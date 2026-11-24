// expect: R0401
use std.strings.{SuffixAutomaton}
let k = SuffixAutomaton<Char>.of("abb").kth_substring(-1) // <- here
print(yes(k == None))

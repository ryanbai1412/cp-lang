// expect: R0401 at 2
use std.strings.{SuffixAutomaton}
let k = SuffixAutomaton<Char>.of("abb").kth_substring(-1)
print(yes(k == None))

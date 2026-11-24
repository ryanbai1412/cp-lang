// expect: ok, output "0 YES"
use std.strings.{SuffixAutomaton}
let m = SuffixAutomaton<Char>.of("abb")
print(m.shortest(0), yes(m.first_pos(0) == None))

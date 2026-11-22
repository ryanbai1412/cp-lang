// expect: ok, output "0 -1"
use std.strings.{SuffixAutomaton}
let m = SuffixAutomaton<Char>.of("abb")
print(m.shortest(0), m.first_pos(0))

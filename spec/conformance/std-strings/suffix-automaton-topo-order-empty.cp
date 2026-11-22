// expect: ok, output "YES"
use std.strings.{SuffixAutomaton}
print(yes(SuffixAutomaton<Char>.of("").topo_order() == [0]))

// expect: ok, output "YES YES YES"
// rule: std.strings.SuffixAutomaton.kth_substring
use std.strings.{SuffixAutomaton}
let sam = SuffixAutomaton<Char>.of("abb")
print(
    yes(sam.kth_substring(3) == Some(['b'])),
    yes(sam.kth_substring(5, distinct: false) == Some(['b', 'b'])),
    yes(sam.kth_substring(6, distinct: false) == None),
)

// expect: ok, output "YES"
use std.strings.{SuffixAutomaton}
print(yes(SuffixAutomaton<Char>.of("abb").kth_substring(5) == None))

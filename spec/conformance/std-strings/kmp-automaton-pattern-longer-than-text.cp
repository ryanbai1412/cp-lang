// expect: ok, output "0"
use std.strings.{KmpAutomaton}
let k = KmpAutomaton<Char>("abc")
print(k.find_all("ab").collect().len)

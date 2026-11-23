// expect: ok, output "128 97 5"
// rule: 02.1.1
use std.strings.{AhoCorasick}
let ac = AhoCorasick<Char>(["ab", "ba"])
print(Char.count, 'a'.ord, ac.len)

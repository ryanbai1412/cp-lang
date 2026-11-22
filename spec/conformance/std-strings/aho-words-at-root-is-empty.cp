// expect: ok, output "0 NO"
use std.strings.{AhoCorasick}
let ac = AhoCorasick<Char>(["ab", "b"])
print(ac.words_at(0).len, yes(ac.hit(0)))

// expect: ok, output "3 patterns, 8 states"
use std.strings.{AhoCorasick}
print(AhoCorasick<Char>(["he", "she", "his"]))

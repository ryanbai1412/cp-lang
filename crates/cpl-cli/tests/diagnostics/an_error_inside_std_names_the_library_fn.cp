use std.strings.{manacher}

domain Pos = 1..=3
let s: [Pos]Char = read()
let pal = manacher(s)
print(yes(pal.is_palindrome(0..=2)))

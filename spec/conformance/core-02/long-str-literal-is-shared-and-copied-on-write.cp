// expect: ok, output "38 38 38 abcdefghijklmnopqrstuvwxyz! abcdefghijklmnopqrstuvwxyz YES 27"
fn say(s: Str) -> Int = s.len
var n = 0
for i in 0..3: n += say("a message past the short-string buffer")
var w = "abcdefghijklmnopqrstuvwxyz"
w += "!"
let again = "abcdefghijklmnopqrstuvwxyz"
print(n / 3, say("a message past the short-string buffer"), "a message past the short-string buffer".len, w, again, yes(again + "!" == w), w.len)

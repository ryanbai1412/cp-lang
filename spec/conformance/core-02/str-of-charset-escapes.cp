// expect: ok, output "3"
// The charset of `Str of` is an ordinary string literal, so `\\` is one backslash (round 23, H24).
let s: Str of "./\\" = "./\\"
print(s.len)

// expect: ok, output "el 2"
// `Str(v)` copies a string view, a slice included, into a new `Str` (round 23, H2).
let s = "hello"
let u = Str(s[1..3])
print(u, u.len)

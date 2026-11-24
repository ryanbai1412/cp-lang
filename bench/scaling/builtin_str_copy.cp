// n: 4194304
// The built-in Str's whole-string copies and scans, at sizes past the caches so each is one O(n) pass.
let n: Int = read()
var t = ""
for i in 0..n: t.push(Char(97 + i * 7919 % 26))
var s = 0
for r in 0..64:
    var c = t.clone() // timed: builtin.Str.clone
    c.fill('q') // timed: builtin.Str.fill
    s += t.to_str().len // timed: builtin.Str.to_str
    if t.contains('A'): s += 1 // timed: builtin.Str.contains
    s += Int(c[r])
print(s)

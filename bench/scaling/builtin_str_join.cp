// The built-in Str's whole-string passes: a join writes 2n chars, find_all scans n.
let n: Int = read()
var t = ""
for i in 0..n: t.push(Char(97 + i * 7919 % 26))
let parts = [Char(97 + i % 26) for i in 0..n]
var s = 0
for _ in 0..64:
    s += ",".join(parts).len // timed: builtin.Str.join
    s += t.find_all("ab").len // timed: builtin.Str.find_all
print(s)

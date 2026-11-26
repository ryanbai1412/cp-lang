// expect: ok, output "hello, world\n12 o YES NO NO\nYES NO\nzb! 7 1 2\n122\n98\n33\nn = 12 hello 3"
// A `Str` is a flat sequence of `Char` (R02.3): literals, `+`, `==`, indexing, a walk,
// and printing next to other values.
fn write_str(x: Int) -> Int = x + 1
let s = "hello"
let t = s + ", world"
print(t)
print(t.len, t[4], yes(s == "hello"), yes(s != "hello"), yes(s == t))
print(yes(1 < 2), yes(2 < 1))
var u = "ab"
u[0] = 'z'
u.push('!')
print(u, 7, [1, 2])
for c in u: print(Int(c))
print("n = {t.len}", s, write_str(2))

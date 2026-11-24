// expect: ok, output "1 2 3 4 50 3 9 hello lo!"
// rule: 05.2.2
// `copy` of a slice view is the owned `List`/`Str` it views: writing and growing it leaves the source alone.
let a = [1, 2, 3, 4]
var b = copy(a[1..3])
b[0] = 50
b.push(9)
let s = "hello"
var t = copy(s[3..5])
t.push('!')
print(a, b, s, t)

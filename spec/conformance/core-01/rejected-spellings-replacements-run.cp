// expect: ok, output "4 1024 3 1"
// rule: 01.11.1
// The "write instead" column of the rejected-spellings table runs.
// a comment
let xs = [1, 2]
let ys = [3]
var t = 0
for x in xs + ys if x % 2 == 1: t += x
let m = ["a": 1]
print(t, 2 ** 10, (xs + ys).len, m["a"])

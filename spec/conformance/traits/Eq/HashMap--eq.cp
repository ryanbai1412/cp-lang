// expect: ok, output "YES NO"
// engines: next
var a: HashMap<Int, Int> = HashMap()
var b: HashMap<Int, Int> = HashMap()
a[1] = 10
a[2] = 20
b[2] = 20
b[1] = 10
let same = a == b
b[1] = 11
print(yes(same), yes(a == b))

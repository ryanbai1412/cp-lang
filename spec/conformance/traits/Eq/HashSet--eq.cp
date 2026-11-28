// expect: ok, output "YES NO"
// engines: next
var a: HashSet<Int> = HashSet()
var b: HashSet<Int> = HashSet()
for x in [1, 2, 3]: a.add(x)
for x in [3, 2, 1]: b.add(x)
let same = a == b
b.add(4)
print(yes(same), yes(a == b))

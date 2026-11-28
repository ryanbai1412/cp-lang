// expect: ok, output "YES NO"
// engines: next
var m: HashMap<Int, Int> = HashMap()
m[5] = 1
print(yes(m.contains(5)), yes(m.contains(6)))

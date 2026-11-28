// expect: ok, output "10"
// engines: next
var m: HashMap<Int, Int> = HashMap()
m[5] = 7
m[5] += 3
print(m[5])

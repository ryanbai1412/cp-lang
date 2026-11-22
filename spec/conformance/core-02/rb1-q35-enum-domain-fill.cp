// expect: ok, output "1"
// rule: 02.7.3
enum Dir = L | R | U | D
var c: [Dir]Int = 0
c[Dir.U] += 1
print(c[Dir.U])

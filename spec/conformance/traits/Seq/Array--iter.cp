// expect: ok, output "0\n10\n20"
domain V = 0..3
var a = [V] v => Int(v) * 10
for x in a: print(x)

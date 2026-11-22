// expect: ok, output "20\n10\n0"
domain V = 0..3
var a = [V] v => Int(v) * 10
for x in a.rev: print(x)

// expect: ok, output "6"
domain R = 0..2
domain C = 0..3
domain D = (R, C)
var k = 0
for x in D.rev:
    k += 1
print(k)

// expect: ok, output "4"
domain D = 0..4
var k = 0
for x in D.rev:
    k += 1
print(k)

// expect: ok, output "3"
let (domain D, comp) = compress([30, 10, 20, 10])
var k = 0
for x in D.rev:
    k += 1
print(k)

// expect: ok, output "4"
domain V = 0..3
var k = 0
for x in V.cuts.rev:
    k += 1
print(k)

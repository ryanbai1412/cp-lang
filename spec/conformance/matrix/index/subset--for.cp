// expect: ok, output "8"
domain B = 0..3
var k = 0
for x in Subset<B>:
    k += 1
print(k)

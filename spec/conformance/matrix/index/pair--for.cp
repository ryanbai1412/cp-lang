// expect: ok, output "6"
domain V = 0..4
var k = 0
for x in Pair<V>:
    k += 1
print(k)

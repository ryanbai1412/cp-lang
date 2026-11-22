// expect: ok, output "4"
enum D = N | E | S | W
var k = 0
for x in D:
    k += 1
print(k)

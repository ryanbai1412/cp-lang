// expect: ok, output "3\n5\n5\n2 6"
// rule: 05.7.1 std.bag.SortedBag.range
// Each range view ends with its loop, expression or block, so the bag can change between them (adversarial round 46).
var b = SortedBag.of([1, 3, 5])
for x in b.range(2..):
    print(x)
b.add(4)
let n = b.range(..4).len
b.add(0)
if b.len > 0:
    let v = b.range(..)
    print(v.len)
b.add(9)
print(n, b.len)

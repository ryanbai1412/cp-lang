// expect: ok, output "1 2"
// reserve is a capacity hint: counts before and after it are kept.
var c = Counter<Int>()
c.add(7, 1)
c.reserve(100)
c.add(3, 2)
print(c[7], c[3])

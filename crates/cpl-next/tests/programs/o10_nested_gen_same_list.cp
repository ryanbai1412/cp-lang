// expect: ok, output "36"
// Legal: two nested walks of the same list, read-only.
var xs: List<Int> = [1, 2, 3]
var s = 0
for x in xs:
    for y in xs: s += x * y
print(s)

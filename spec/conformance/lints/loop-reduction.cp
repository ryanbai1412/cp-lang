// expect: L0504
let xs: List<Int> = [1, 2, 3]
var total = 0 // <- here
for x in xs:
    total += x
print(total)

// expect: L0501
let xs: List<Int> = [1, 4, 7]
var found = false // <- here
for x in xs:
    if x > 3:
        found = true
        break
print(found)

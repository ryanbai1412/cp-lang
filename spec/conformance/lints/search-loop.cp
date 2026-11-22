// expect: L0501 at 2
let xs: List<Int> = [1, 4, 7]
var found = false
for x in xs:
    if x > 3:
        found = true
        break
print(found)

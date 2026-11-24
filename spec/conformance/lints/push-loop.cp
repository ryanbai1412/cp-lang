// expect: L0507
let raw = [3, 1, 2]
var doubled: List<Int> = [] // <- here
for x in raw: doubled.push(2 * x)
print(doubled)

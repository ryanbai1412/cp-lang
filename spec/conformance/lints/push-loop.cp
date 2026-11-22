// expect: L0507 at 2
let raw = [3, 1, 2]
var doubled: List<Int> = []
for x in raw: doubled.push(2 * x)
print(doubled)

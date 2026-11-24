// expect: E0602
// rule: 06.2.1
var xs = [1, 2]
let grow = () => xs.push(3)
for x in xs:
    grow() // <- here
print(xs.len)

// expect: L0606 at 1
// rule: 03.1.8
var a: [1000000]Int = [1000000].filled(0)
a[0] = 7
print(a[0])

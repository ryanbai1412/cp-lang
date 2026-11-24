// expect: L0607
var a: [200000](w: Int, b: Bool) = [200000].filled((w: 0, b: false)) // <- here
a[0] = (w: 7, b: true)
print(a[0].w)

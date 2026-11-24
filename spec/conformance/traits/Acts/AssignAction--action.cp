// expect: ok, output "6"
var a = SegTree(+, [1, 2, 3], act: assign)
let two: ?Int = Some(2)
a.apply(0..3, two)
print(a.all)

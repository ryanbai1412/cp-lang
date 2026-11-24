// expect: ok, output "11"
var f = SegTree(+, [1, 2, 3], act: affine)
f.apply(0..2, (2, 1))
print(f.all)

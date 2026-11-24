// expect: ok, output "12"
var s = SegTree(+, [1, 2, 3], act: add)
s.apply(0..2, 3)
print(s.all)

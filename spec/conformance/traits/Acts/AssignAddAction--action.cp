// expect: ok, output "17"
var a = SegTree(+, [1, 2, 3], act: assign_add)
a.apply(0..2, (set: Some(4), add: 1))
a.apply(1..3, (set: None, add: 2))
print(a.all)

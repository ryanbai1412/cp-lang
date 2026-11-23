// expect: ok, output "6 3"
domain D = 0..3
let a: [D]Int = [1, 2, 3]
let s = SegTree<D, Int>(+, a)
let m = SegTree<D, Int>(max, a)
print(s.all, m.all)

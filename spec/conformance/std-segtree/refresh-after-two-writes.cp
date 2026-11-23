// expect: ok, output "14 23"
domain D = 0..3
var w: [D]Int = [1, 2, 3]
var s = SegTree<D, Int>.of(+, i => w[i])
w[D(1)] = 10
s.refresh(D(1))
let a = s.all
w[D(0)] = 10
w[D(2)] = 3
s.refresh(D(0))
print(a, s.all)

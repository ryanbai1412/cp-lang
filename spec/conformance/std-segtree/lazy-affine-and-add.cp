// expect: ok, output "15 27 24 21 4"
// rule: std.segtree SegTree(op, a, act:) lazy range updates with affine, add, assign
domain I = 1..=5
var st = SegTree(+, [I] i => i.ord + 1, act: affine)
let s0 = st.all
st.apply(2..=4, (1, 3))
st.apply(1..=3, (0, 5))
let s1 = st.fold(1..=5)

var st_add = SegTree(+, [I] i => i.ord + 1, act: add)
st_add.apply(2..=4, 3)
let s2 = st_add.all

var st_asgn = SegTree(+, [I] i => i.ord + 1, act: assign)
st_asgn.apply(1..=3, Some(4))
let s3 = st_asgn.all
let g1 = st_asgn.get(I.nth(0))
print(s0, s1, s2, s3, g1)

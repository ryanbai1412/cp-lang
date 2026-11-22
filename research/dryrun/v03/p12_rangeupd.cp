// CSES 1735 Range Updates and Sums: range add, range assign, range sum.
let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
let t = read<[I]1..=1e6>()
var st = SegTree(+, [I] i => Int(t[i]), act: affine)
for _ in 0..q:
    let kind = read<1..=3>()
    let (a, b) = read<(I, I)>()
    if kind == 3:
        print(st.fold(a..=b))
    else:
        let x = read<1..=1e6>()
        st.apply(a..=b, if kind == 1: (1, x) else: (0, x))

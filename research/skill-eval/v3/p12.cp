let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
type Val = 1..=1e6
let a = read<[V]Val>()
var st = SegTree(+, a, act: affine)
for _ in 0..q:
    let kind = read<1..=3>()
    let (l, r) = read<(V, V)>()
    if kind == 1:
        st.apply(l..=r, (1, read<Val>()))
    elif kind == 2:
        st.apply(l..=r, (0, read<Val>()))
    else:
        print(st.fold(l..=r))

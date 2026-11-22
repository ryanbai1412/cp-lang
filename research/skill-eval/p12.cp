let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain I = 1..=n
let a = read<[I]Int>()
var st = BeatsSegTree<I>(a)
for _ in 0..q:
    let (k, l, r) = read<(1..=3, I, I)>()
    if k == 1:
        let x = read<Int>()
        st.add(l..=r, x)
    elif k == 2:
        let x = read<Int>()
        st.assign(l..=r, x)
    else:
        print(st.sum(l..=r))

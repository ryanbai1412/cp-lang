let (n, q): (1..=2e5, 1..=2e5) = read()
domain V = 1..=n
let a: [V]0..=1e9 = read()
var st = SegTree(min, a)
for _ in 0..q:
    let t: 1..=2 = read()
    if t == 1:
        let (i, x): (V, 0..=1e9) = read()
        st.set(i, x)
    else:
        let (l, r): (V, V) = read()
        print(st.fold(l..=r))

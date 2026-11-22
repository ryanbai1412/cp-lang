let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain V = 1..=n
type Val = 1..=1e9
var sums = Fenwick(read<[V]Val>())
for _ in 0..q:
    if read<1..=2>() == 1:
        let (k, u) = read<(V, Val)>()
        sums.set(k, u)
    else:
        let (l, r) = read<(V, V)>()
        print(sums.fold(l..=r))

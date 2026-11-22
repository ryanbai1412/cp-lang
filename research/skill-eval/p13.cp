let (n, q) = read<(1..=2e5, 1..=2e5)>()
domain I = 1..=n
let a = read<[I]Int>()
var fw = Fenwick<I, Int>(a)
for _ in 0..q:
    let k = read<1..=2>()
    if k == 1:
        let (pos, val) = read<(I, Int)>()
        fw.set(pos, val)
    else:
        let (l, r) = read<(I, I)>()
        print(fw.fold(l..=r))

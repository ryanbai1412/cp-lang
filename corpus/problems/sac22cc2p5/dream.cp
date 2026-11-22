// sac22cc2p5: point assignments, and sums of every other element from l to r (those of l's parity).
// One Fenwick per parity, each holding only its own positions.
let (n, q) = read<(1..=1_000_000, 1..=1_000_000)>()
domain I = 1..=n
let a = read<[I]Int>()
var half = [0..2] p => Fenwick([I] i => if i % 2 == p: a[i] else: 0)
for _ in 0..q:
    if read<1..=2>() == 1:
        let (i, x) = read<(I, Int)>()
        half[i % 2].set(i, x)
    else:
        let (l, r) = read<(I, I)>()
        print(half[l % 2].sum(l..=r))

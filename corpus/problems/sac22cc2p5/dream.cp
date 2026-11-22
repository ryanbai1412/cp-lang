// sac22cc2p5: point assignments, and sums of every other element from l to r (those of l's parity).
// One Fenwick per parity, each holding only its own positions.
let (n, q) = read<(1..=1_000_000, 1..=1_000_000)>()
domain I = 1..=n
let a = read<[I]Int>()
fn odd(i: I) -> Bool = Int(i) % 2 == 1
var half = [Bool] p => Fenwick([I] i => if odd(i) == p: a[i] else: 0)
enum Ask:
    @tag("1") Set(i: I, x: Int)
    @tag("2") Sum(l: I, r: I)
for ask in read<[q]Ask>():
    match ask:
        Set(i, x) => half[odd(i)].set(i, x)
        Sum(l, r) => print(half[odd(l)].sum(l..=r))

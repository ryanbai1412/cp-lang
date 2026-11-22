// SAC 22 CC2 P5: point set, and "sum of the elements in l..=r at the same parity as l".
// Keep one Fenwick per parity.
let (n, q) = read (Int, Int)
domain I = 1..=n
let a = read [I]Int
var by = [for p in 0..2: Fenwick(+, [I] i => if Int(i) % 2 == p: a[i] else: 0)]
enum Q: Set(i: I, x: Int) = 1, Ask(r: Range<I>) = 2
for query in read [q]Q:
    match query:
        Set(i, x): by[Int(i) % 2].set(i, x)
        Ask(r): print(by[Int(r.first!) % 2].fold(r))

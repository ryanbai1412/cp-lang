// SAC 22 CC2 P5: point set, and "sum of the elements in l..=r at the same parity as l".
// Keep one Fenwick per parity.
let n: 1..=1_000_000 = read()
let q: 1..=1_000_000 = read()
domain I = 1..=n
domain Parity = 0..2
fn side(i: I) -> Parity = Parity(i.ord % 2)
let a: [I]-1e9..=1e9 = read()
var by = [Parity] p => Fenwick([I] i => if side(i) == p: a[i] else: 0)
enum Q = @tag(1) Put(i: I, x: -1e9..=1e9) | @tag(2) Ask(r: Range<I>)
for query in read<[q]Q>():
    match query:
        Put(i, x) => by[side(i)].set(i, x)
        Ask(r) => print(by[side(r.first!)].fold(r))

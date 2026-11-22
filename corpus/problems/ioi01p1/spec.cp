// ioi01p1 Mobile Phones: point additions and rectangle sums on an s by s grid, until command 3.
let (_, s) = read<(0..=0, 1..=1_024)>()
domain X = 0..s
var phones = Fenwick2D<X, X, Int>()
enum Cmd:
    @tag("1") Add(x: X, y: X, a: Int)
    @tag("2") Ask(l: X, b: X, r: X, t: X)
    @tag("3") Stop
while true:
    match read<Cmd>():
        Add(x, y, a) => phones.add(x, y, a)
        Ask(l, b, r, t) => print(phones.fold(l..=r, b..=t))
        Stop => break

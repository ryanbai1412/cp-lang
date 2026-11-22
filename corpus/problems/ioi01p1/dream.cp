// ioi01p1 Mobile Phones: point additions and rectangle sums on an s by s grid, until command 3.
let (_, s) = read<(0..=0, 1..=1_024)>()
domain X = 0..s
var phones = Fenwick2D<X, X, Int>()
while true:
    let cmd = read<1..=3>()
    if cmd == 1:
        let (x, y, a) = read<(X, X, Int)>()
        phones.add(x, y, a)
    elif cmd == 2:
        let (l, b, r, t) = read<(X, X, X, X)>()
        print(phones.sum(l..=r, b..=t))
    else:
        break

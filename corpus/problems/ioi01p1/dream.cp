// IOI 2001 Mobile Phones: an S×S grid of counters; commands add to one cell or ask the sum of a
// rectangle, until command 3. A 2D Fenwick tree answers both in O(log² S).
let (_, n): (0..=0, 1..=1024) = read()
domain X = 0..n
var phones = Fenwick2D<X, X, Int>.new()
while true:
    match read<Int>():
        1 =>
            let (x, y, a): (X, X, -32767..=32767) = read()
            phones.add(x, y, a)
        2 =>
            let (l, b, r, t): (X, X, X, X) = read()
            print(phones.fold(l..=r, b..=t))
        _ => break

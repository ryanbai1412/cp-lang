// IOI 2001 Mobile Phones: an S×S grid of counters; commands add to one cell or ask the sum of a
// rectangle, until command 3.
enum Cmd: Size(s: Int) = 0, Add(x: Int, y: Int, a: Int) = 1, Sum(l: Int, b: Int, r: Int, t: Int) = 2, End = 3
let Size(s) = read Cmd
domain X = [s]
var phones = Fenwick(+, [X, X]0)
for cmd in read stream Cmd until End:
    match cmd:
        Add(x, y, a): phones.add((X(x), X(y)), a)
        Sum(l, b, r, t): print(phones.fold((X(l)..=X(r), X(b)..=X(t))))

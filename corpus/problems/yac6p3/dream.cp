// YAC 6 P3 (interactive): two hidden cells in an n×m grid; a query returns the distance to the
// nearer one. Four corner queries give x+y and x−y bounds for each; one more query picks the
// right pairing.
@interactive
let (n, m) = read (Int, Int)
domain Cell = (1..=n, 1..=m)
fn ask(c: (Int, Int)) -> Int = query("? {c.0} {c.1}", read Int)
let (c1, c2, c3, c4) = (ask((1, 1)) + 2, ask((n, 1)) - n + 1, ask((1, m)) - m + 1, ask((n, m)) - n - m)
let first = ((c1 + c2) / 2).then(y => (c1 - y, y))
let (p, q) = if Cell.at(first.swap()) is some(_) and ask(first.swap()) == 0:
        (first.swap(), ((c3 + c4) / -2).then(y => (-c4 - y, y)).swap())
    else:
        let x1 = (c1 + c3) / 2
        let x2 = (c2 + c4) / -2
        ((x1, c1 - x1), (x2, -c4 - x2))
print("! {p.0} {p.1} {q.0} {q.1}")

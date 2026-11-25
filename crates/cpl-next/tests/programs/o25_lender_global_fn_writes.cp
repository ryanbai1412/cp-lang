// expect: E0405
// The value assigned into a lent slot comes from a fn that writes the same global struct.
struct C:
    v: Int
    n: Int
    fn slot(mut self) -> lends mut Int:
        lend mut v
        n += 1  // <- here
var c = C(1, 0)
fn poke() -> Int:
    c.v = 50
    7
c.slot() += poke()
print(c.v, c.n)

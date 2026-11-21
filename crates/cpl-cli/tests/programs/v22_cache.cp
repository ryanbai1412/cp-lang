// A method that takes plain `self` and appends to its struct's own list
// works on the caller's struct, so what it worked out is kept: `Comb<P>`
// grows its tables this way behind a `let`.
use std.mod.{Mod, Comb}

struct Squares:
    known: List<Int>

    fn fill(self, n: Int):
        while known.len <= n: known.push(known.len * known.len)

    fn at(self, n: Int) -> Int:
        self.fill(n)
        known[n]

fn tenth(s: Squares) -> Int:
    s.at(10)

let s = Squares([])
print(s.at(3), s.known.len)
print(s.at(2), s.known.len)
print(tenth(s), s.known.len)
print(Squares([]).at(4))

type Z = Mod<1000000007>
let c = Comb<Z>()
print(c.choose(10, 3).v, c.choose(30, 15).v, c.fact(5).v, c.choose(3, 5).v)
print(Comb<Z>().choose(4, 2).v)

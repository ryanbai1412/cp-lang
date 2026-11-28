// expect: ok, output "4 0 3"
// rule: R03.1.4
// A field typed `0..P` by a value parameter is an `Int` range in its own methods: it reads as an
// `Int`, and a store into it is checked.
struct Ring<const P: Int>(pub v: 0..P):
    pub fn new(x: Int) -> Ring<P> = Ring(x % P)
    pub fn next(self) -> Ring<P> = Ring((v + 1) % P)
    pub fn twice_past(self) -> Int = v * 2 - P
let a = Ring<5>(3).next()
print(a.v, a.next().v, a.twice_past())

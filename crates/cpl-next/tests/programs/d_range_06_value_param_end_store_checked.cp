// expect: R0201
// rule: R03.1.4
// A store into a field typed `0..P` by a value parameter is checked against `P` as it runs.
struct Ring<P>(pub v: 0..P):
    pub fn new(x: Int) -> Ring<P> = Ring(x % P)
    pub fn bump(self) -> Ring<P> = Ring<P>(v + 1)
print(Ring<5>(3).bump().v)
print(Ring<5>(4).bump().v)

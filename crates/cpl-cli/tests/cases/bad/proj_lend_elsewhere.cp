// error: may only lend part of `self`
let n: 2..=9 = read()
domain V = 1..=n
var glob: [V]Int = 0
struct Bag:
    cnt: [V]Int
    fn at(self, i: V) lends mut Int:
        lend mut glob[i]

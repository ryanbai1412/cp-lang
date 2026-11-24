// A runtime modulus from a fn-local `let` (`type ZQ = Mod(q)`) works as at top level and in a loop.
use std.mod.{Comb}
fn t():
    let q = 1_000_000_007
    type ZQ = Mod(q)
    print(ZQ(q + 3))
    let cq = Comb<ZQ>(10)
    print(cq.choose(10, 3))
    for p in [2, 12]:
        type Z = Mod(p)
        print(Z(7) * Z(5))
t()

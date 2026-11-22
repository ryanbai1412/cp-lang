// Array literal `[e0, e1]` coerced to a plain-enum-indexed `[Side]T` array
// in `let`, struct fields, and function arguments (dream/agc077a:74).
enum Side = X | Y
struct Run(l: Int, r: Int)
struct Block(seen: [Side]Run, real: [Side]Run)

fn total(b: Block) -> Int =
    b.seen[X].l + b.seen[Y].r + b.real[X].r + b.real[Y].l

let a: [Side]Int = [10, 20]
let r1 = Run(1, 5)
let r2 = Run(2, 6)
let blk = Block(seen: [r1, r2], real: [r2, r1])
print(a[X], a[Y], total(blk))

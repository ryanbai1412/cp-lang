use std.derive.{Gen, gen, gen_fields}
use std.rand.Rng

struct Cell(r: Int, c: Int)
@allow(unread_field)
struct Query(at: Cell, on: Bool, tag: Char)

impl Gen for Cell:
    fn gen(mut r: Rng) -> Cell = gen_fields(mut r)

var rng = Rng.new(read())
let n: Int = gen(mut rng)
let q: Query = gen_fields(mut rng)
let qs = [gen_fields<Query>(mut rng) for _ in 0..3]
print(n)
print(q)
for x in qs:
    print(x.at.r + x.at.c, x.on, x.tag)

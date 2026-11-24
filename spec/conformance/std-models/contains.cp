// expect: ok, output "ok"
// input: "7 300\n"
// model: contains
// `x in s` through std's `contains` against a hand loop over seeded random Deques and a user
// Seq of -4..=4 (length 0..=6), for every x in -5..=5. Reads `seed n`; prints the first miss.
use std.rand.{Rng}
use std.deque.{Deque}

struct Bag(xs: List<Int>)
impl Seq<Int> for Bag:
    fn iter(self) -> yields Int:
        for x in xs: yield x

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for _ in 0..n:
    if bad != "": break
    let len = rng.int(0, 6)
    let xs = [rng.int(-4, 4) for _ in 0..len]
    var dq = Deque<Int>()
    for x in xs: dq.push(x)
    let bag = Bag([x for x in xs])
    for x in -5..=5:
        var want = false
        for y in xs:
            if y == x: want = true
        if (x in dq) != want or (x in bag) != want:
            bad = "{xs} {x}"
print(if bad == "": "ok" else: bad)

// expect: ok, output "ok"
// input: "1 300\n"
// model: fold_from
// model: reduce
// model: fold
// `fold(op)`, `reduce(f)` and `fold_from(init, f)` against a hand loop over seeded random lists
// of -9..=9 (length 0..=6): `+`, `max` and `Monoid` concatenation (not commutative, so a fold out
// of order shows) for `fold`, `-` for `reduce`, and a base-10 accumulator for `fold_from`.
// Reads `seed n`; prints the first list where they differ.
use std.rand.{Rng}
use std.algebra.{Monoid}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let cat = Monoid<Str>("", (a: Str, b: Str) => a + b)
var bad = ""
for _ in 0..n:
    if bad != "": break
    let len = rng.int(0, 6)
    let xs = [rng.int(-9, 9) for _ in 0..len]
    var sum = 0
    var big = Int.MIN
    var word = ""
    var digits = 0
    var diff: ?Int = None
    for x in xs:
        sum += x
        big = max(big, x)
        word = word + "{x}"
        digits = digits * 10 + x
        diff = Some(if let d = diff: d - x else: x)
    let words = ["{x}" for x in xs]
    if xs.fold(+) != sum: bad = "fold(+) on {xs}"
    elif xs.fold(max) != big: bad = "fold(max) on {xs}"
    elif words.fold(cat) != word: bad = "fold(cat) on {xs}"
    elif xs.reduce((a, b) => a - b) != diff: bad = "reduce(-) on {xs}"
    elif xs.fold_from(0, (a, x) => a * 10 + x) != digits: bad = "fold_from on {xs}"
print(if bad == "": "ok" else: bad)

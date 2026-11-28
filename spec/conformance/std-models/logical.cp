// expect: ok, output "ok"
// input: "11 300\n"
// model: logical_and
// model: logical_or
// std's `logical_and` and `logical_or` (the values `and` and `or`) against the truth table,
// and their folds over seeded random Bool lists (length 0..=6) against a hand loop. Reads
// `seed n`; prints the first miss.
use std.rand.{Rng}
use std.ops.{logical_and, logical_or}

let seed: Int = read()
let n: Int = read()
var rng = Rng(seed)
var bad = ""
for a in [false, true]:
    for b in [false, true]:
        if bad == "" and logical_and(a, b) != (a and b): bad = "logical_and"
        elif bad == "" and logical_or(a, b) != (a or b): bad = "logical_or"
for k in 0..n:
    if bad != "": break
    let len = rng.int(0, 6)
    let xs = [rng.int(0, 1) == 1 for _ in 0..len]
    var all = true
    var any = false
    for x in xs:
        if not x: all = false
        if x: any = true
    if xs.fold(and) != all: bad = "case {k}: fold(and)"
    elif xs.fold(or) != any: bad = "case {k}: fold(or)"
    elif xs.fold(logical_and) != all: bad = "case {k}: fold(logical_and)"
print(if bad == "": "ok" else: bad)

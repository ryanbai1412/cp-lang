// expect: ok, output "ok"
// input: "1 300\n"
// model: read_fields
// model: eq_fields
// model: lt_fields
// model: show_fields
// model: hash_fields
use std.rand.{Rng}
use std.derive.{eq_fields, hash_fields, lt_fields, read_fields, show_fields}

struct Header(seed: Int, n: Int)
struct P(a: Int, s: Str, c: Int)

// splitmix64's finalizer with the odd constants `Hash` uses, written out.
fn mix(z0: U64) -> U64:
    var z = z0 + U64(-7046029254386353131)
    z = (z ^ (z >> 30)) * U64(-4658895280553007687)
    z = (z ^ (z >> 27)) * U64(-7723592293110705685)
    z ^ (z >> 31)

fn str_hash(s: Str) -> U64:
    var h = U64(s.len)
    for ch in s: h = mix(h ^ U64(Int(ch)))
    h

fn want_hash(p: P) -> U64 = mix(mix(mix(U64(0) ^ mix(U64(p.a))) ^ str_hash(p.s)) ^ mix(U64(p.c)))

fn want_lt(x: P, y: P) -> Bool =
    if x.a != y.a: x.a < y.a elif x.s != y.s: x.s < y.s else: x.c < y.c

let h = read_fields<Header>()
var rng = Rng(h.seed)
let words = ["", "a", "ab", "b", "ba"]
var bad = ""
for k in 0..h.n:
    if bad != "": break
    let x = P(rng.int(-2, 2), words[rng.int(0, 4)], rng.int(-2, 2))
    let y = if rng.chance(0.3): P(x.a, x.s, x.c) else: P(rng.int(-2, 2), words[rng.int(0, 4)], rng.int(-2, 2))
    let same = x.a == y.a and x.s == y.s and x.c == y.c
    if eq_fields(x, y) != same: bad = "op {k}: eq_fields({show_fields(x)}, {show_fields(y)})"
    elif lt_fields(x, y) != want_lt(x, y): bad = "op {k}: lt_fields({show_fields(x)}, {show_fields(y)})"
    elif show_fields(x) != "{x.a} {x.s} {x.c}": bad = "op {k}: show_fields = {show_fields(x)}"
    elif hash_fields(x) != want_hash(x): bad = "op {k}: hash_fields({show_fields(x)}) = {hash_fields(x)} want {want_hash(x)}"
    elif same and hash_fields(x) != hash_fields(y): bad = "op {k}: equal values hash apart"
print(if bad == "": "ok" else: bad)

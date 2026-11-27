// expect: ok, output "ok"
// input: "1 300\n"
// model: raw_empty
// model: raw_new
// model: raw_get
// model: raw_set
// model: raw_take
// model: raw_cap
// model: check_key
// model: check_range
// std.raw's cells against a List<Int>: seeded random raw_set, raw_get and raw_take on 16 cells
// from raw_new, and 16 cells from raw_cap, each set before it is read, compared after each op.
// check_key and check_range pass a true. Reads `seed n`; prints the first op that differs.
use std.raw.{raw_empty, raw_new, raw_get, raw_set, raw_take, raw_cap, check_key, check_range}

/// A Lehmer generator: the case's own, so it needs no `std.rand`.
struct Lcg:
    var x: Int

impl Lcg:
    fn int(mut self, lo: Int, hi: Int) -> Int:
        x = x * 48271 % 2147483647
        lo + x % (hi - lo + 1)

let seed0 = read<Int>()
let n = read<Int>()
var rng = Lcg(seed0 % 2147483646 + 1)
var r = raw_new(16, 7)
var a = List.filled(16, 7)
var c: Raw<Int> = raw_cap(16)
for i in 0..16: raw_set(mut c, i, i * i)
let e: Raw<Str> = raw_empty()
var bad = ""
if false: bad = raw_get(e, 0)
var at = 0
for k in 0..n:
    let i = rng.int(0, 15)
    let op = rng.int(0, 2)
    if op == 0:
        let v = rng.int(-100, 100)
        raw_set(mut r, i, v)
        a[i] = v
    elif op == 1:
        if raw_get(r, i) != a[i]: bad = "get"
    else:
        let v = raw_take(mut r, i)
        if v != a[i]: bad = "take"
        raw_set(mut r, i, v)
    check_key(0 <= i and i < 16)
    check_range(i <= 16)
    if raw_get(c, i) != i * i: bad = "cap"
    if bad != "":
        at = k
        break
if bad == "": print("ok") else: print("op", at, bad)

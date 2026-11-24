// expect: ok, output "ok"
// input: "1 300\n"
// model: ctz
// model: clz
// model: bit_width
// model: bit_ceil
// std.math's bit counts against loops over the word's bits: seeded words, mostly small,
// sometimes any 64 bits or 0. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var bad = ""
for op in 0..n:
    let x = if rng.chance(0.1): 0 elif rng.chance(0.4): rng.int(Int.MIN, Int.MAX) else: rng.int(-100, 5000)
    var low = 64
    var high = -1
    for i in 0..64:
        if x.bit(i):
            if low == 64: low = i
            high = i
    if ctz(x) != low or ctz(U64(x)) != low: bad = "op {op}: ctz({x})"
    elif clz(x) != 63 - high or clz(U64(x)) != 63 - high: bad = "op {op}: clz({x})"
    elif bit_width(x) != high + 1 or bit_width(U64(x)) != high + 1: bad = "op {op}: bit_width({x})"
    elif x <= 1 << 62:
        var p = 1
        while p < x: p *= 2
        if bit_ceil(x) != p: bad = "op {op}: bit_ceil({x})"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)

// The VM reads every spec without a call in it itself, its bounds and
// lengths evaluated in registers: words, floats, bounded ints, tuples,
// optionals and arrays whose length was read before.
let n: 1..=10 = read()
let (lo, hi): (Int, Int) = read()
let xs: [0..n]1..=100 = read()
let ts: [0..n]?Int = read(none: -1)
var sum = 0
var absent = 0
for i in 0..n:
    let v: lo..=hi = read()
    sum += xs[i] + v
    if ts[i] == None: absent += 1
let w: Str = read()
let f: F64 = read()
print(sum, absent, w, f * 2.0)

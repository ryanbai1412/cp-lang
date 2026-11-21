// Assign each value 1..=845 a distinct 30-bit query mask with bit 0 used
// <= 300 times (one type-4 query) and each bit 1..29 used <= 60 times
// (29 type-2 queries). The 30 query responses give each position a unique
// signature that directly decodes the permutation.
@!interactive

let (masks, mask_to_val) = do:
    var cap: [0..30]Int = 60
    cap[0] = 300
    var ms: [1..=845]Int = 0
    var inv = Map<Int, Int>()
    var k = 0

    fn try_add(bits: List<Int>) -> Bool:
        let m = sum(1 << b for b in bits)
        if inv.contains(m) or any(bits, b => cap[b] == 0): return false
        k += 1
        ms[k] = m
        inv[m] = k
        for b in bits: cap[b] -= 1
        true

    try_add([])
    for i in 0..30: try_add([i])
    for i in 0..30:
        for j in i + 1..30: try_add([i, j])
    for d in 1..=9:
        for x in 0..29: try_add([0, 1 + x, 1 + (x + d) % 29])
    for x in 0..9: try_add([0, 1 + x, 1 + (x + 10) % 29])

    while k < 845:
        var order = List(1..30)
        sort(mut order, by: b => desc(cap[b]))
        first([(x, y, z) for x in order for y in order for z in order where x < y and y < z],
              t => try_add([t.0, t.1, t.2]))
    (ms, inv)

--- multitest ---
let n: 2..=845 = read()
domain Pos = 1..=n
var sig: [Pos]Int = 0

for q in 0..30:
    let vals = [v for v in 1..=n where masks[v].bit(q)]
    if vals.is_empty: continue
    let qtype = if q == 0: 4 else: 2
    for p in ask<[vals.len]Pos>(qtype, vals.len, vals): sig[p] |= 1 << q

print("!", [mask_to_val[sig[p]] for p in Pos])

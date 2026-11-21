@!interactive

// Build 845 distinct 30-bit query masks with bit 0 used <= 300 times
// and each bit 1..29 used <= 60 times (no backtracking or RNG needed).
var cap: [0..30]Int = 60
cap[0] = 300
var masks: [1..=845]Int = 0
var num_masks = 0
var mask_to_val = Map<Int, Int>()

fn try_add(bits: List<Int>) -> Bool:
    var m = 0
    for b in bits: m |= 1 << b
    if mask_to_val.contains(m): return false
    for b in bits:
        if cap[b] == 0: return false
    num_masks += 1
    masks[num_masks] = m
    mask_to_val[m] = num_masks
    for b in bits: cap[b] -= 1
    true

try_add([])
for i in 0..30: try_add([i])
for i in 0..30:
    for j in i + 1..30:
        try_add([i, j])
for d in 1..=9:
    for x in 0..29:
        try_add([0, 1 + x, 1 + (x + d) % 29])
for x in 0..9:
    try_add([0, 1 + x, 1 + (x + 10) % 29])

while num_masks < 845:
    var order = List(1..30)
    sort(mut order, by: b => desc(cap[b]))
    var added = false
    for x in order:
        for y in order:
            for z in order:
                if not added and x < y and y < z and try_add([x, y, z]):
                    added = true

let t: 1..=40 = read()
for _ in 0..t:
    let n: 2..=845 = read()
    domain Pos = 1..=n
    var sig: [Pos]Int = 0

    for q in 0..30:
        let vals = [v for v in 1..=n where masks[v].bit(q)]
        if vals.len == 0: continue
        let qtype = if q == 0: 4 else: 2
        print(qtype, vals.len, vals)
        let resp: [0..vals.len]Pos = read()
        for p in resp:
            sig[p] |= 1 << q

    let ans = [mask_to_val[sig[p]] for p in Pos]
    print("!", ans)

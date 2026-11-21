// Codeforces 2129 F2: Permutation (Hard Version).
// Assign each value 1..=890 a 29-bit mask (each bit used <= 60 times):
// 18 unique triples plus two copies of all 436 subsets of size <= 2.
// 29 type-2 queries recover every position's mask, narrowing each to a
// unique value or a colliding pair (v0, v1) with v0 <= 454 < v1. One
// randomized type-3 probe on one position per colliding pair resolves all pairs.
@!interactive

var masks: [1..=890]Int = 0
var num_masks = 0
var mask_vals = Map<Int, List<Int>>()

fn add_mask(m: Int):
    num_masks += 1
    masks[num_masks] = m
    if mask_vals.contains(m):
        mask_vals[m].push(num_masks)
    else:
        mask_vals[m] = [num_masks]

for i in 0..9:
    add_mask((1 << (3 * i)) | (1 << (3 * i + 1)) | (1 << (3 * i + 2)))
    add_mask((1 << (3 * i)) | (1 << ((3 * i + 4) % 27)) | (1 << ((3 * i + 8) % 27)))

for _ in 0..2:
    add_mask(0)
    for i in 0..29:
        add_mask(1 << i)
        for j in i + 1..29:
            add_mask((1 << i) | (1 << j))

var rng = 50
fn next_bit() -> Int:
    rng = (rng * 1_103_515_245 + 12_345) % 2_147_483_648
    (rng / 65536) % 2

let t: 1..=40 = read()
for _ in 0..t:
    let n: 2..=890 = read()
    domain Pos = 1..=n
    var sig: [Pos]Int = 0

    for q in 0..29:
        let vals = [v for v in 1..=n if masks[v].bit(q)]
        if vals.len == 0: continue
        print(2, vals.len, vals)
        let resp: [0..vals.len]Pos = read()
        for p in resp:
            sig[p] |= 1 << q

    var by_sig = Map<Int, List<Pos>>()
    for p in Pos:
        let s = sig[p]
        if by_sig.contains(s):
            by_sig[s].push(p)
        else:
            by_sig[s] = [p]

    var ans: [Pos]Int = 0
    var swap_pair: [1..=n]?(Pos, Pos) = none
    var probes = List<Pos>()

    for (s, ps) in by_sig:
        let vs = mask_vals[s]
        if ps.len == 1:
            ans[ps[0]] = vs[0]
        else:
            var (p0, p1) = (ps[0], ps[1])
            if next_bit() == 1:
                swap(mut p0, mut p1)
            ans[p0] = vs[0]
            ans[p1] = vs[1]
            probes.push(p0)
            swap_pair[vs[1]] = (p0, p1)

    if probes.len > 0:
        print(3, probes.len, probes)
        let resp_len = min(probes.len, 300)
        let resp: [0..resp_len](1..=n) = read()
        for v in resp:
            if let (p0, p1) = swap_pair[v]:
                ans.swap(p0, p1)

    print("!", ans)

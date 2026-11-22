// Codeforces 2222 E — Interactive Bitwise Operation & Mask Recovery
//
// Starting from an empty set, inserting 0 leaves size 1 iff the hidden
// operation is k == 1 (in which case we also insert `(1 << n) - 1` so `v`
// becomes detectable). Next, recover the hidden n-bit mask `v` from MSB to
// LSB via `ask('Q', v + (1 << b)) > 0`. Finally, when `k in {2, 3}`, one extra
// insertion (using `v & (v - 1)` if `v` is not a power of 2, or `nv` = `3` /
// `v + 1` plus a query if `v` is a power of 2) distinguishes `k == 2` from `3`.

@!interactive

fn ask(kind: Char, x: Int) -> Int:
    print(kind, x)
    read()

@!multitest
let n: 1..=20 = read()
print(0)

let n_sz = ask('I', 0)
if n_sz == 1: ask('I', (1 << n) - 1)

var v = 0
for b in (0..n).rev:
    if ask('Q', v + (1 << b)) > 0: v += 1 << b

let k = if n_sz == 1:
    1
elif not v.is_pow2:
    if ask('I', v & (v - 1)) == 3: 3 else: 2
else:
    let nv = if v == 1: 3 else: v + 1
    ask('I', nv)
    if ask('Q', nv) != 1: 3 else: 2

print("A", k, v)

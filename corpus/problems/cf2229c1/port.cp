// Codeforces 2229 C1 — Right-to-Left Prefix Sign Flip
//
// Sweep `i` from `n` down to `1` (`domain Pos = 1..=n`), tracking `flip` =
// parity of flips already applied to suffixes starting after `i`. Whenever
// `a[i]` is positive under `flip` (`(x > 0) != flip`), flip prefix `1..=i`.
// Treating `x == 0` as non-positive is safe because `-0 == 0` (flipping or not
// leaves `0` valid as `<= 0`).

@!multitest
let n: 1..=200_000 = read()
domain Pos = 1..=n
let a: [Pos]-10 ** 9..=10 ** 9 = read()

var ops: List<Pos> = []
var flip = false
for i in Pos.rev:
    if (a[i] > 0) != flip:
        ops.push(i)
        flip = not flip
print(ops.len)
print(ops)

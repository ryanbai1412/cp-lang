// Matched benchmark: atcoder_dp_s's digit DP, `next[Rem.wrap(r.ord + x)] += below[r]` over
// 200000 digits and d = 97: a wrap must cost one remainder, like the C++ `(r + x) % d`.

type Z = Mod<1_000_000_007>
let d = 97
domain Rem = 0..d
var below: [Rem]Z = 0
below[Rem(0)] = Z(1)
for _ in 0..200_000:
    var next: [Rem]Z = 0
    for r in Rem:
        for x in 0..=9: next[Rem.wrap(r.ord + x)] += below[r]
    below = next
print(below[Rem(0)])

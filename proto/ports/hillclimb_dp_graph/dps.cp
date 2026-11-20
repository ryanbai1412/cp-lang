// AtCoder DP S Digit Sum: count the integers in 1..=K whose digit sum is a
// multiple of d. K has up to 10000 digits.
type Z = Mod<1_000_000_007>
let k: Str = read()
let d: 1..=100 = read()
domain Rem = 0..d

// below[r]: numbers strictly below the prefix of K read so far (same length,
// leading zeros allowed) whose digit sum is r mod d. `prefix` is K's own.
var below: [Rem]Z = 0
var prefix = Rem.first
for c in k:
    var next: [Rem]Z = 0
    for r in Rem:
        for x in 0..=9: next[r.cyc(x)] += below[r]
    for x in 0..c - '0': next[prefix.cyc(x)] += 1
    below = next
    prefix = prefix.cyc(c - '0')
print(below[Rem.first] + Z(Int(prefix == Rem.first)) - 1)   // zero is not in 1..=K

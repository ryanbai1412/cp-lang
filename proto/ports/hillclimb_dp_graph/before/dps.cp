// cp: 0.1 0f8d77b+dirty
// AtCoder DP S, Digit Sum: count the integers in 1..=K whose digit sum is a
// multiple of d. K has up to 10000 digits.
const P = 1000000007
type Z = Mod<P>
domain Digit = '0'..='9'
let k: [domain Pos]Digit = read()
let d: 1..=100 = read()
domain Rem = 0..d

// below[r]: numbers strictly below the prefix of K read so far (same length,
// leading zeros allowed) whose digit sum is r mod d. `prefix` is K's own.
var below: [Rem]Z = Z(0)
var prefix = Rem.first
for c in k:
    var next: [Rem]Z = Z(0)
    for r in Rem:
        for x in Digit:
            next[Rem((r.idx + x.idx) % d)] += below[r]
    for x in Digit:
        if x < c: next[Rem((prefix.idx + x.idx) % d)] += Z(1)
    below = next
    prefix = Rem((prefix.idx + c.idx) % d)
let upto = below[Rem.first] + (if prefix == Rem.first: Z(1) else: Z(0))
print(upto - Z(1))                          // zero is not in 1..=K

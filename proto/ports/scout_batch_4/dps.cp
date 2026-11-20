// Educational DP S, Digit Sum: count the integers in 1..=k whose digit sum is
// a multiple of d. k has up to 10^4 digits.
// original: corpus/submissions/4131864.cpp
const P = 1000000007
type Z = Mod<P>
domain Digit = '0'..='9'
let k: [domain Pos]Digit = read()
let d: 1..=100 = read()

// below[r]: numbers already smaller than k's prefix, with digit sum r mod d.
// The prefix of k itself is the one number still tied, with digit sum `tied`.
type Row = [0..d]Z
var below: Row = 0
var tied = 0
for p in Pos:
    var next: Row = 0
    for r in 0..d:
        for x in Digit: next[(r + x.idx) % d] += below[r]
    for x in Digit.first..k[p]: next[(tied + x.idx) % d] += 1
    tied = (tied + k[p].idx) % d
    below = next
// 0 was counted and is not in 1..=k
print(below[0] + (if tied == 0: 1 else: 0) - 1)

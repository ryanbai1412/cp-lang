// NOIP 2018 Currency System: the smallest set of coins that pays exactly the
// same amounts. A coin is needed iff the smaller coins cannot make it.
@!multitest
let n: 1..=100 = read()
let coins: [0..n]1..=25000 = read()
let top = max(coins)

var payable: [0..=top]Bool = false
payable[0] = true
var needed = 0
for c in sorted(coins) if not payable[c]:
    needed += 1
    for a in c..=top: payable[a] |= payable[a - c]
print(needed)

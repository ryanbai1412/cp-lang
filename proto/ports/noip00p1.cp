// cp: 0.1 1f59ac0+dirty
// NOIP 2000 Base Conversion: write x in a negative base -r. With Euclidean
// division the remainder is already a digit, whatever the sign of the base.
let x: -32768..=32767 = read()
let base: -20..=-2 = read()

var digits: List<Char> = []
var left = x
while left != 0:
    let d = left % base
    digits.push(if d < 10: '0' + d else: 'A' + (d - 10))
    left /= base
print([d for d in digits.rev])

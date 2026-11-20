// NOIP 2000 Base Conversion: write x in a negative base. Euclidean division
// leaves a remainder that is already a digit, whatever the sign of the base.
let (x, base): (-32768..=32767, -20..=-2) = read()

var (digits, left) = ("", x)
while left != 0:
    let d = left % base
    digits = "{if d < 10: '0' + d else: 'A' + (d - 10)}{digits}"
    left /= base
print(digits)

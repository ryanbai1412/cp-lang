// NOIP 2000 Base Conversion: write x in a negative base. Euclidean division
// leaves a remainder that is already a digit, whatever the sign of the base.
let (x, base): (-32768..=32767, -20..=-2) = read()

var (digits, left) = ("", x)
while left != 0:
    digits = "0123456789ABCDEFGHIJ"[left % base] + digits
    left /= base
print(digits)

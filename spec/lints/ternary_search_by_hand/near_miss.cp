let n: 1..=1000000 = read()
var x = n
var digits = 0
while x > 0:
    x = (x - x % 3) / 3
    digits += 1
print(digits)

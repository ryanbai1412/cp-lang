@!multitest
let x: 1..=1000000000000000000 = read()
var s = 0
var y = x
while y > 0:  // expect: manual_digit_loop
    s += y % 10
    y /= 10
print(s)

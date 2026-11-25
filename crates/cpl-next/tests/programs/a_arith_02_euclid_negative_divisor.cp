// expect: ok, output "1 1 1 0"
// rule: R01.5.2: 7 / -2 == -3, 7 % -2 == 1
// Signs of / and %: floor division and a remainder with the divisor's sign?  7 / -2, 7 % -2, -7 % 2, 0 / -5.
print(7 / -2 + 4, 7 % -2, -7 % 2, 0 / -5)

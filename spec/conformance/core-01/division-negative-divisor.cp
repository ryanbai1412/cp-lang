// expect: ok, output "-3 1 -4 1"
// rule: 01.5.2
// `/` and `%` are Euclidean: `%` is never negative, so a negative divisor rounds `/` up (rebuild 8, H-DIV-SEMANTICS).
print(7 / -2, 7 % -2, -7 / 2, -7 % 2)

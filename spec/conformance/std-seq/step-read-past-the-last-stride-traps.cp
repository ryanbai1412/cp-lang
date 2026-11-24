// expect: R0101
// rule: R09.3.10
// Reading a step view at a key between strides traps as an index does, after a read on the stride succeeds.
let a = [10, 11, 12, 13, 14, 15, 16]
let s = a.step(3)
print(s[3])
print(s[4]) // <- here

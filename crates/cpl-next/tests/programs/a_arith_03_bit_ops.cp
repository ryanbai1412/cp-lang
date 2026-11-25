// expect: ok, output "2"
// rule: R01.5.9
// xor on Ints (R01.5.9): 3 ^ 1 == 2; and 6 & 3 == 2; 2 | 0 == 2.
let a = 3 ^ 1
let b = 6 & 3
let c = 2 | 0
print(if a == 2 and b == 2 and c == 2: 2 else: 0)

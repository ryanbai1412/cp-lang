// expect: ok, output "7 5 3 1\n3 2 8\n2 5 8"
// rule: 01.8.4
// A stepped range is a range value: it reverses and has O(1) len/first/last (rebuild round 15 H-STEP-RANGE).
print((1..=7 by 2).rev)
let r = 2..9 by 3
print(r.len, r.first!, r.last!)
print(r)

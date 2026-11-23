// expect: E0314 at 5
// rule: 01.7.3
// A guarded arm never counts toward exhaustiveness, even `_ if ...` (rebuild round 16 H-GUARD-EXHAUSTIVE).
enum E = A | B | C
let e = E.A
let x = 0
let r = match e:
    _ if x == 0 => 1
    A => 2
    B => 3
print(r)

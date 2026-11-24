// expect: E0203
// rule: 07.7.4
// has: "chain("
// Concatenation is the free fn `chain(a, b)` (R07.7.4); the error for `.chain` names it (rebuild round 29 L3).
let s = "abcd"
print(s[2..].chain(s[..2].rev)) // <- here

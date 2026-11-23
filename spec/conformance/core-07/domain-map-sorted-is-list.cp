// expect: ok, output "1 1 2 3"
// rule: 07.11.1
// `sorted()` on a mapped domain gives a `List`, keyed from 0 (rebuild 7, H-DOMAIN-MAP-SORTED).
domain I = 1..=3
let s = I.map(i => 4 - Int(i)).sorted()
print(s[0], s)

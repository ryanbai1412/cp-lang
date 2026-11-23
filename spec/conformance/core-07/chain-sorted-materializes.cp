// expect: ok, output "1 2 3"
// rule: 07.7.4
// `chain(a, b)` is a `Seq`; `.sorted()` collects it, no `List(...)` first (rebuild round 6 H-GEN-SORTED).
let a = [3, 1]
let b = [2]
print(chain(a, b).sorted())

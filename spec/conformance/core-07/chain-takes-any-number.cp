// expect: ok, output "1 2 3 4 5"
// gap: stdlib 2026-11-24T05Z
// rule: 07.7.4
// `chain` takes any number of sequences, as free `min` and `max` take any number of values (rebuild round 32 FL5).
let a = [1, 2]
print(List(chain(a, [3], [4, 5])))

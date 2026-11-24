// expect: ok, output "0 -1\nYES NO"
// rule: 07.9.2
// `and` and `or` have identities (`true`, `false`), so their folds are `Bool` and an empty one is the identity; `max` on `Int` has none (adversarial round 46).
let none: List<Int> = []
print(none.fold(+), none.fold(max) ?? -1)
let nb: List<Bool> = []
print(yes(nb.fold(and)), yes(nb.fold(or)))

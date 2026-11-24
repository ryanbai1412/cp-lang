// expect: R0301
// rule: 02.4.8
domain V = 1..=3
var depth = [V] v => if v == V(2): None else: Some(1)
let full = depth! // <- here
print(full[V(1)])

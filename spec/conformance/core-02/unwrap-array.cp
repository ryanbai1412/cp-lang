// expect: R0301 at 3
// rule: 02.4.8
domain V = 1..=3
var depth = [V] v => if v == V(2): None else: Some(1)
let full = depth!
print(full[V(1)])

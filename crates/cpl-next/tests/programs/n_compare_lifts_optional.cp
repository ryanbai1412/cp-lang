// expect: ok, output "1 0 1\n1 1"
// rule: R04.4.22 R04.4.23
// A comparison's other operand fixes the type: a plain value against an optional one is lifted
// to `Some`, whichever side it is on.
domain V = 0..3
let from: ?V = None
let at: ?V = Some(V(2))
let u = V.first!
let a = if u != from: 1 else: 0
let b = if u == at: 1 else: 0
let c = if at == V(2): 1 else: 0
print(a, b, c)
let k: ?Int = Some(4)
print(if 4 == k: 1 else: 0, if k != 5: 1 else: 0)

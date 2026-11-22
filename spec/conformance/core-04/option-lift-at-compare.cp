// expect: ok, output "1"
// rule: 04.4.22 04.4.23
domain V = 0..3
let from: ?V = None
let u = V.first!
if u != from: print(1) else: print(0)

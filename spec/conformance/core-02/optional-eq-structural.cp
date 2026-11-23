// expect: ok, output "1 0"
// rule: 02.4.5
// `?T` is not `Ord` but `==` is structural: `None == None` (rebuild round 18 H15).
let a: ?Int = None
let b: ?Int = None
let c: ?Int = Some(1)
print(if a == b: 1 else: 0, if a == c: 1 else: 0)

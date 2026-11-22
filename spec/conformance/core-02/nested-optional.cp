// expect: ok, output "1 1"
// rule: 02.4.3
let a: ?(?Int) = Some(None)
let b: ?(?Int) = None
print(if a is Some(None): 1 else: 0, if b is None: 1 else: 0)
